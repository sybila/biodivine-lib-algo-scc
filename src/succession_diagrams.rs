use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// One "node" of a succession diagram: a trap space of that node,
/// plus the stable motifs that percolate to node successors.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct SdEntry {
    trap: HashMap<String, u8>,
    motifs: Vec<HashMap<String, u8>>,
}

/// Load succession diagram data from a file path.
pub fn load_sd_data_from_file(path: &str) -> Vec<SdEntry> {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[cfg(test)]
mod tests {
    use crate::succession_diagrams::SdEntry;

    #[test]
    pub fn test_deserialize() {
        let raw_data = r#"
            [
                {"trap": {}, "motifs": [{"v_ICL": 0}]}, 
                {"trap": {"v_MUS81": 0, "v_FAN1": 0, "v_FANCM": 0, "v_FAcore": 0, "v_ICL": 0, "v_XPF": 0, "v_FANCD2I": 0}, 
                 "motifs": [{"v_ADD": 0, "v_FAN1": 0, "v_FANCD2I": 0, "v_FANCM": 0, "v_FAcore": 0, "v_ICL": 0, "v_MUS81": 0, "v_XPF": 0}, {"v_DSB": 0, "v_FAN1": 0, "v_FANCD2I": 0, "v_FANCM": 0, "v_FAcore": 0, "v_ICL": 0, "v_MUS81": 0, "v_XPF": 0}]}, 
                {"trap": {"v_ADD": 0, "v_XPF": 0, "v_ICL": 0, "v_FANCM": 0, "v_FANCD2I": 0, "v_USP1": 0, "v_FAN1": 0, "v_PCNATLS": 0, "v_MUS81": 0, "v_FAcore": 0}, 
                 "motifs": [{"v_ADD": 0, "v_DSB": 0, "v_FAN1": 0, "v_FANCD2I": 0, "v_FANCM": 0, "v_FAcore": 0, "v_ICL": 0, "v_MUS81": 0, "v_PCNATLS": 0, "v_USP1": 0, "v_XPF": 0}]}, 
                {"trap": {"v_FAN1": 0, "v_ICL": 0, "v_FANCD1N": 0, "v_DSB": 0, "v_ssDNARPA": 0, "v_FAcore": 0, "v_MRN": 0, "v_KU": 0, "v_DNAPK": 0, "v_NHEJ": 0, "v_MUS81": 0, "v_BRCA1": 0, "v_FANCD2I": 0, "v_FANCM": 0, "v_XPF": 0, "v_HRR": 0, "v_H2AX": 0, "v_FANCJBRCA1": 0, "v_RAD51": 0}, 
                 "motifs": [{"v_ADD": 0, "v_BRCA1": 0, "v_DNAPK": 0, "v_DSB": 0, "v_FAN1": 0, "v_FANCD1N": 0, "v_FANCD2I": 0, "v_FANCJBRCA1": 0, "v_FANCM": 0, "v_FAcore": 0, "v_H2AX": 0, "v_HRR": 0, "v_ICL": 0, "v_KU": 0, "v_MRN": 0, "v_MUS81": 0, "v_NHEJ": 0, "v_RAD51": 0, "v_XPF": 0, "v_ssDNARPA": 0}, {"v_ATM": 0, "v_ATR": 0, "v_BRCA1": 0, "v_DNAPK": 0, "v_DSB": 0, "v_FAN1": 0, "v_FANCD1N": 0, "v_FANCD2I": 0, "v_FANCJBRCA1": 0, "v_FANCM": 0, "v_FAcore": 0, "v_H2AX": 0, "v_HRR": 0, "v_ICL": 0, "v_KU": 0, "v_MRN": 0, "v_MUS81": 0, "v_NHEJ": 0, "v_RAD51": 0, "v_XPF": 0, "v_ssDNARPA": 0}]}, 
                {"trap": {"v_DSB": 0, "v_HRR": 0, "v_ADD": 0, "v_NHEJ": 0, "v_FAcore": 0, "v_USP1": 0, "v_KU": 0, "v_DNAPK": 0, "v_MUS81": 0, "v_PCNATLS": 0, "v_BRCA1": 0, "v_H2AX": 0, "v_MRN": 0, "v_FANCD1N": 0, "v_FANCJBRCA1": 0, "v_ICL": 0, "v_FANCD2I": 0, "v_FANCM": 0, "v_RAD51": 0, "v_ssDNARPA": 0, "v_XPF": 0, "v_FAN1": 0}, 
                 "motifs": [{"v_ADD": 0, "v_ATM": 0, "v_ATR": 0, "v_BRCA1": 0, "v_DNAPK": 0, "v_DSB": 0, "v_FAN1": 0, "v_FANCD1N": 0, "v_FANCD2I": 0, "v_FANCJBRCA1": 0, "v_FANCM": 0, "v_FAcore": 0, "v_H2AX": 0, "v_HRR": 0, "v_ICL": 0, "v_KU": 0, "v_MRN": 0, "v_MUS81": 0, "v_NHEJ": 0, "v_PCNATLS": 0, "v_RAD51": 0, "v_USP1": 0, "v_XPF": 0, "v_ssDNARPA": 0}]}, 
                {"trap": {"v_NHEJ": 0, "v_FANCM": 0, "v_DNAPK": 0, "v_ICL": 0, "v_RAD51": 0, "v_FANCD1N": 0, "v_XPF": 0, "v_ATR": 0, "v_p53": 0, "v_ssDNARPA": 0, "v_CHK2": 0, "v_MUS81": 0, "v_CHK1": 0, "v_ATM": 0, "v_FAN1": 0, "v_KU": 0, "v_DSB": 0, "v_MRN": 0, "v_H2AX": 0, "v_FAcore": 0, "v_FANCD2I": 0, "v_BRCA1": 0, "v_HRR": 0, "v_FANCJBRCA1": 0}, 
                 "motifs": [{"v_ADD": 0, "v_ATM": 0, "v_ATR": 0, "v_BRCA1": 0, "v_CHK1": 0, "v_CHK2": 0, "v_DNAPK": 0, "v_DSB": 0, "v_FAN1": 0, "v_FANCD1N": 0, "v_FANCD2I": 0, "v_FANCJBRCA1": 0, "v_FANCM": 0, "v_FAcore": 0, "v_H2AX": 0, "v_HRR": 0, "v_ICL": 0, "v_KU": 0, "v_MRN": 0, "v_MUS81": 0, "v_NHEJ": 0, "v_RAD51": 0, "v_XPF": 0, "v_p53": 0, "v_ssDNARPA": 0}]}, 
                {"trap": {"v_XPF": 0, "v_p53": 0, "v_ssDNARPA": 0, "v_KU": 0, "v_FANCD1N": 0, "v_CHK1": 0, "v_MUS81": 0, "v_DSB": 0, "v_ATR": 0, "v_FANCJBRCA1": 0, "v_H2AX": 0, "v_RAD51": 0, "v_ADD": 0, "v_BRCA1": 0, "v_FAN1": 0, "v_FANCD2I": 0, "v_FANCM": 0, "v_ICL": 0, "v_HRR": 0, "v_DNAPK": 0, "v_MRN": 0, "v_CHK2": 0, "v_PCNATLS": 0, "v_NHEJ": 0, "v_USP1": 0, "v_FAcore": 0, "v_ATM": 0}, 
                 "motifs": []}
            ]
        "#;
        
        let data: Vec<SdEntry> = serde_json::from_str(raw_data).unwrap();
        assert_eq!(data.len(), 7);
    }
    
}