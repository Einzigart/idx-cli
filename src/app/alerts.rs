use crate::app::{App, InputMode, ViewMode};
use crate::config::{Alert, AlertType};

impl App {
    pub fn open_alert_modal(&mut self) {
        let symbol = match self.view_mode {
            ViewMode::Watchlist => self.selected_watchlist_symbol(),
            ViewMode::Portfolio => self.selected_portfolio_symbol(),
            ViewMode::News => return,
        };
        if let Some(sym) = symbol {
            self.alert_symbol = Some(sym);
            self.alert_list_selected = 0;
            self.input_mode = InputMode::AlertList;
        } else {
            self.status_message = Some("No symbol selected".to_string());
        }
    }

    pub fn close_alert_modal(&mut self) {
        self.alert_symbol = None;
        self.alert_list_selected = 0;
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
    }

    pub fn alert_list_up(&mut self) {
        if self.alert_list_selected > 0 {
            self.alert_list_selected -= 1;
        }
    }

    pub fn alert_list_down(&mut self) {
        let count = self
            .alert_symbol
            .as_ref()
            .map(|s| self.config.alerts_for_symbol(s).len())
            .unwrap_or(0);
        if self.alert_list_selected < count {
            self.alert_list_selected += 1;
        }
    }

    pub fn alert_list_confirm(&mut self) {
        let sym = match &self.alert_symbol {
            Some(s) => s.clone(),
            None => return,
        };
        let count = self.config.alerts_for_symbol(&sym).len();
        if self.alert_list_selected == count {
            // "Add" row selected — start the add wizard
            self.pending_alert_type = AlertType::Above;
            self.input_buffer.clear();
            self.input_mode = InputMode::AlertAddType;
        } else {
            // Toggle enable/disable on the selected existing alert
            let id = self.config.alerts_for_symbol(&sym)[self.alert_list_selected]
                .id
                .clone();
            self.config.toggle_alert(&id);
            if let Err(e) = self.config.save() {
                self.status_message = Some(format!("Save error: {}", e));
            }
        }
    }

    pub fn alert_list_delete(&mut self) -> anyhow::Result<()> {
        let sym = match &self.alert_symbol {
            Some(s) => s.clone(),
            None => return Ok(()),
        };
        let count = self.config.alerts_for_symbol(&sym).len();
        if self.alert_list_selected < count {
            let id = self.config.alerts_for_symbol(&sym)[self.alert_list_selected]
                .id
                .clone();
            self.config.remove_alert(&id);
            self.config.save()?;
            if self.alert_list_selected > 0
                && self.alert_list_selected >= self.config.alerts_for_symbol(&sym).len()
            {
                self.alert_list_selected -= 1;
            }
            self.status_message = Some("Alert deleted".to_string());
        }
        Ok(())
    }

    pub fn alert_type_up(&mut self) {
        self.pending_alert_type = self.pending_alert_type.prev();
    }

    pub fn alert_type_down(&mut self) {
        self.pending_alert_type = self.pending_alert_type.next();
    }

    pub fn alert_type_confirm(&mut self) {
        self.input_buffer.clear();
        self.input_mode = InputMode::AlertAddValue;
    }

    pub fn alert_value_confirm(&mut self) -> anyhow::Result<()> {
        if let Ok(val) = self.input_buffer.trim().parse::<f64>() {
            if val > 0.0 {
                if let Some(ref sym) = self.alert_symbol {
                    let alert = Alert::new(sym, self.pending_alert_type.clone(), val);
                    self.config.add_alert(alert);
                    self.config.save()?;
                    self.status_message = Some(format!("Alert added for {}", sym));
                    let count = self.config.alerts_for_symbol(sym).len();
                    self.alert_list_selected = count.saturating_sub(1);
                }
            } else {
                self.status_message = Some("Value must be > 0".to_string());
            }
        } else {
            self.status_message = Some("Invalid number".to_string());
        }
        self.input_buffer.clear();
        self.input_mode = InputMode::AlertList;
        Ok(())
    }

    pub fn cancel_alert_add(&mut self) {
        self.input_buffer.clear();
        self.input_mode = InputMode::AlertList;
    }

    pub fn check_alerts(&mut self) -> Vec<(String, String)> {
        let mut triggered: Vec<(String, String)> = Vec::new();

        let to_trigger: Vec<(String, String, String)> = self
            .config
            .alerts
            .iter()
            .filter_map(|alert| {
                let quote = self.quotes.get(&alert.symbol)?;
                if alert.should_trigger(quote.price, quote.change_percent) {
                    let msg = match alert.alert_type {
                        AlertType::Above => {
                            format!("{} crossed above {:.0}", alert.symbol, alert.target_value)
                        }
                        AlertType::Below => {
                            format!("{} crossed below {:.0}", alert.symbol, alert.target_value)
                        }
                        AlertType::PercentGain => format!(
                            "{} up {:.2}% (target +{:.2}%)",
                            alert.symbol, quote.change_percent, alert.target_value
                        ),
                        AlertType::PercentLoss => format!(
                            "{} down {:.2}% (target -{:.2}%)",
                            alert.symbol, quote.change_percent, alert.target_value
                        ),
                    };
                    Some((alert.id.clone(), alert.symbol.clone(), msg))
                } else {
                    None
                }
            })
            .collect();

        for (id, symbol, msg) in to_trigger {
            self.config.mark_triggered(&id);
            triggered.push((symbol, msg));
        }

        if !triggered.is_empty() {
            let _ = self.config.save();
        }

        triggered
    }
}
