use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct QSO {
    pub call: String,
    pub gridsquare: String,
    pub mode: String,
    pub submode: String,
    pub rst_sent: String,
    pub rst_rcvd: String,
    pub qso_date: String,
    pub time_on: String,
    pub qso_date_off: String,
    pub time_off: String,
    pub band: String,
    pub freq: String,
    pub freq_rx: String,
    pub operator: String,
    pub comment: String,
    pub power: String,
    pub my_gridsquare: String,
    pub station_callsign: String,
}

impl QSO {
    pub fn new() -> Self {
        QSO {
            call: String::new(),
            gridsquare: String::new(),
            mode: String::new(),
            submode: String::new(),
            rst_sent: String::new(),
            rst_rcvd: String::new(),
            qso_date: String::new(),
            time_on: String::new(),
            qso_date_off: String::new(),
            time_off: String::new(),
            band: String::new(),
            freq: String::new(),
            freq_rx: String::new(),
            operator: String::new(),
            comment: String::new(),
            power: String::new(),
            my_gridsquare: String::new(),
            station_callsign: String::new(),
        }
    }

    pub fn from_map(map: &HashMap<String, String>) -> Self {
        QSO {
            call: map.get("call").cloned().unwrap_or_default(),
            gridsquare: map.get("gridsquare").cloned().unwrap_or_default(),
            mode: map.get("mode").cloned().unwrap_or_default(),
            submode: map.get("submode").cloned().unwrap_or_default(),
            rst_sent: map.get("rst_sent").cloned().unwrap_or_default(),
            rst_rcvd: map.get("rst_rcvd").cloned().unwrap_or_default(),
            qso_date: map.get("qso_date").cloned().unwrap_or_default(),
            time_on: map.get("time_on").cloned().unwrap_or_default(),
            qso_date_off: map.get("qso_date_off").cloned().unwrap_or_default(),
            time_off: map.get("time_off").cloned().unwrap_or_default(),
            band: map.get("band").cloned().unwrap_or_default(),
            freq: map.get("freq").cloned().unwrap_or_default(),
            freq_rx: map.get("freq_rx").cloned().unwrap_or_default(),
            operator: map.get("operator").cloned().unwrap_or_default(),
            comment: map.get("comment").cloned().unwrap_or_default(),
            power: map.get("power").cloned().unwrap_or_default(),
            my_gridsquare: map.get("my_gridsquare").cloned().unwrap_or_default(),
            station_callsign: map.get("station_callsign").cloned().unwrap_or_default(),
        }
    }

    pub fn from_adif(input: &str) -> Vec<Self> {
        let mut qsos = Vec::new();
        let mut current = HashMap::new();
        let re = Regex::new(r"(?i)<([a-z_]+)(?::(\d+))?(?::([a-z]+))?>([^<]*)").unwrap();
        for cap in re.captures_iter(input) {
            let field = cap[1].to_uppercase();
            let value = cap[4].trim().to_string();
            if field == "EOR" {
                qsos.push(Self::from_map(&current));
                current.clear();
                continue;
            }
            if field == "EOH" {
                current.clear();
                continue;
            }
            current.insert(field.to_lowercase(), value);
        }
        qsos
    }
    
    pub fn to_adif(&self) -> String {
        let mut adif = String::new();
        if !self.call.is_empty() { adif.push_str(&format!("<CALL:{}>{}", self.call.len(), self.call)); }
        if !self.gridsquare.is_empty() { adif.push_str(&format!("<GRIDSQUARE:{}>{}", self.gridsquare.len(), self.gridsquare)); }
        if !self.mode.is_empty() { adif.push_str(&format!("<MODE:{}>{}", self.mode.len(), self.mode)); }
        if !self.submode.is_empty() { adif.push_str(&format!("<SUBMODE:{}>{}", self.submode.len(), self.submode)); }
        if !self.rst_sent.is_empty() { adif.push_str(&format!("<RST_SENT:{}>{}", self.rst_sent.len(), self.rst_sent)); }
        if !self.rst_rcvd.is_empty() { adif.push_str(&format!("<RST_RCVD:{}>{}", self.rst_rcvd.len(), self.rst_rcvd)); }
        if !self.qso_date.is_empty() { adif.push_str(&format!("<QSO_DATE:{}>{}", self.qso_date.len(), self.qso_date)); }
        if !self.time_on.is_empty() { adif.push_str(&format!("<TIME_ON:{}>{}", self.time_on.len(), self.time_on)); }
        if !self.qso_date_off.is_empty() { adif.push_str(&format!("<QSO_DATE_OFF:{}>{}", self.qso_date_off.len(), self.qso_date_off)); }
        if !self.time_off.is_empty() { adif.push_str(&format!("<TIME_OFF:{}>{}", self.time_off.len(), self.time_off)); }
        if !self.band.is_empty() { adif.push_str(&format!("<BAND:{}>{}", self.band.len(), self.band)); }
        if !self.freq.is_empty() { adif.push_str(&format!("<FREQ:{}>{}", self.freq.len(), self.freq)); }
        if !self.freq_rx.is_empty() { adif.push_str(&format!("<FREQ_RX:{}>{}", self.freq_rx.len(), self.freq_rx)); }
        if !self.operator.is_empty() { adif.push_str(&format!("<OPERATOR:{}>{}", self.operator.len(), self.operator)); }
        if !self.comment.is_empty() { adif.push_str(&format!("<COMMENT:{}>{}", self.comment.len(), self.comment)); }
        if !self.power.is_empty() { adif.push_str(&format!("<POWER:{}>{}", self.power.len(), self.power)); }
        if !self.my_gridsquare.is_empty() { adif.push_str(&format!("<MY_GRIDSQUARE:{}>{}", self.my_gridsquare.len(), self.my_gridsquare)); }
        if !self.station_callsign.is_empty() { adif.push_str(&format!("<STATION_CALLSIGN:{}>{}", self.station_callsign.len(), self.station_callsign)); }
        adif.push_str("<EOR>\r\n");
        adif
    }
}
