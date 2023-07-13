use crate::unicode_table;
use rand::rngs::ThreadRng;
use rand::Rng;
use rand_regex;
use std::char::from_u32;
use std::ops::Range;

static FORBIDDEN_CODEPOINTS: [u32; 14] = [
    0x0022, 0x0223, 0x0025, 0x003C, 0x003E, 0x005B, 0x005C, 0x005D, 0x005E, 0x0060, 0x007B, 0x007C,
    0x007D, 0x007F,
];
static FORBIDDEN_RANGES: [Range<u32>; 2] = [(0xD800..0xE000), (0xFDD0..0xFDF0)];
static NONCHARACTER_MASK: u32 = 0x0FFFE;

//static UNICODE_CODEPOINTS: Vec<(u32, String)> = get_unicode_codepoints();

pub struct CodepointGenerator {
    regex: rand_regex::Regex,
}
static UNICODE_LETTERS: &str = r"[\xAA\xB5\xBA\xC0-\xD6\xD8-\xF6\xF8-\u02C1\u02C6-\u02D1\u02E0-\u02E4\u02EC\u02EE\u0370-\u0374\u0376\u0377\u037A-\u037D\u037F\u0386\u0388-\u038A\u038C\u038E-\u03A1\u03A3-\u03F5\u03F7-\u0481\u048A-\u052F\u0531-\u0556\u0559\u0560-\u0588\u05D0-\u05EA\u05EF-\u05F2\u0620-\u064A\u066E\u066F\u0671-\u06D3\u06D5\u06E5\u06E6\u06EE\u06EF\u06FA-\u06FC\u06FF\u0710\u0712-\u072F\u074D-\u07A5\u07B1\u07CA-\u07EA\u07F4\u07F5\u07FA\u0800-\u0815\u081A\u0824\u0828\u0840-\u0858\u0860-\u086A\u08A0-\u08B4\u08B6-\u08C7\u0904-\u0939\u093D\u0950\u0958-\u0961\u0971-\u0980\u0985-\u098C\u098F\u0990\u0993-\u09A8\u09AA-\u09B0\u09B2\u09B6-\u09B9\u09BD\u09CE\u09DC\u09DD\u09DF-\u09E1\u09F0\u09F1\u09FC\u0A05-\u0A0A\u0A0F\u0A10\u0A13-\u0A28\u0A2A-\u0A30\u0A32\u0A33\u0A35\u0A36\u0A38\u0A39\u0A59-\u0A5C\u0A5E\u0A72-\u0A74\u0A85-\u0A8D\u0A8F-\u0A91\u0A93-\u0AA8\u0AAA-\u0AB0\u0AB2\u0AB3\u0AB5-\u0AB9\u0ABD\u0AD0\u0AE0\u0AE1\u0AF9\u0B05-\u0B0C\u0B0F\u0B10\u0B13-\u0B28\u0B2A-\u0B30\u0B32\u0B33\u0B35-\u0B39\u0B3D\u0B5C\u0B5D\u0B5F-\u0B61\u0B71\u0B83\u0B85-\u0B8A\u0B8E-\u0B90\u0B92-\u0B95\u0B99\u0B9A\u0B9C\u0B9E\u0B9F\u0BA3\u0BA4\u0BA8-\u0BAA\u0BAE-\u0BB9\u0BD0\u0C05-\u0C0C\u0C0E-\u0C10\u0C12-\u0C28\u0C2A-\u0C39\u0C3D\u0C58-\u0C5A\u0C60\u0C61\u0C80\u0C85-\u0C8C\u0C8E-\u0C90\u0C92-\u0CA8\u0CAA-\u0CB3\u0CB5-\u0CB9\u0CBD\u0CDE\u0CE0\u0CE1\u0CF1\u0CF2\u0D04-\u0D0C\u0D0E-\u0D10\u0D12-\u0D3A\u0D3D\u0D4E\u0D54-\u0D56\u0D5F-\u0D61\u0D7A-\u0D7F\u0D85-\u0D96\u0D9A-\u0DB1\u0DB3-\u0DBB\u0DBD\u0DC0-\u0DC6\u0E01-\u0E30\u0E32\u0E33\u0E40-\u0E46\u0E81\u0E82\u0E84\u0E86-\u0E8A\u0E8C-\u0EA3\u0EA5\u0EA7-\u0EB0\u0EB2\u0EB3\u0EBD\u0EC0-\u0EC4\u0EC6\u0EDC-\u0EDF\u0F00\u0F40-\u0F47\u0F49-\u0F6C\u0F88-\u0F8C\u1000-\u102A\u103F\u1050-\u1055\u105A-\u105D\u1061\u1065\u1066\u106E-\u1070\u1075-\u1081\u108E\u10A0-\u10C5\u10C7\u10CD\u10D0-\u10FA\u10FC-\u1248\u124A-\u124D\u1250-\u1256\u1258\u125A-\u125D\u1260-\u1288\u128A-\u128D\u1290-\u12B0\u12B2-\u12B5\u12B8-\u12BE\u12C0\u12C2-\u12C5\u12C8-\u12D6\u12D8-\u1310\u1312-\u1315\u1318-\u135A\u1380-\u138F\u13A0-\u13F5\u13F8-\u13FD\u1401-\u166C\u166F-\u167F\u1681-\u169A\u16A0-\u16EA\u16F1-\u16F8\u1700-\u170C\u170E-\u1711\u1720-\u1731\u1740-\u1751\u1760-\u176C\u176E-\u1770\u1780-\u17B3\u17D7\u17DC\u1820-\u1878\u1880-\u1884\u1887-\u18A8\u18AA\u18B0-\u18F5\u1900-\u191E\u1950-\u196D\u1970-\u1974\u1980-\u19AB\u19B0-\u19C9\u1A00-\u1A16\u1A20-\u1A54\u1AA7\u1B05-\u1B33\u1B45-\u1B4B\u1B83-\u1BA0\u1BAE\u1BAF\u1BBA-\u1BE5\u1C00-\u1C23\u1C4D-\u1C4F\u1C5A-\u1C7D\u1C80-\u1C88\u1C90-\u1CBA\u1CBD-\u1CBF\u1CE9-\u1CEC\u1CEE-\u1CF3\u1CF5\u1CF6\u1CFA\u1D00-\u1DBF\u1E00-\u1F15\u1F18-\u1F1D\u1F20-\u1F45\u1F48-\u1F4D\u1F50-\u1F57\u1F59\u1F5B\u1F5D\u1F5F-\u1F7D\u1F80-\u1FB4\u1FB6-\u1FBC\u1FBE\u1FC2-\u1FC4\u1FC6-\u1FCC\u1FD0-\u1FD3\u1FD6-\u1FDB\u1FE0-\u1FEC\u1FF2-\u1FF4\u1FF6-\u1FFC\u2071\u207F\u2090-\u209C\u2102\u2107\u210A-\u2113\u2115\u2119-\u211D\u2124\u2126\u2128\u212A-\u212D\u212F-\u2139\u213C-\u213F\u2145-\u2149\u214E\u2183\u2184\u2C00-\u2C2E\u2C30-\u2C5E\u2C60-\u2CE4\u2CEB-\u2CEE\u2CF2\u2CF3\u2D00-\u2D25\u2D27\u2D2D\u2D30-\u2D67\u2D6F\u2D80-\u2D96\u2DA0-\u2DA6\u2DA8-\u2DAE\u2DB0-\u2DB6\u2DB8-\u2DBE\u2DC0-\u2DC6\u2DC8-\u2DCE\u2DD0-\u2DD6\u2DD8-\u2DDE\u2E2F\u3005\u3006\u3031-\u3035\u303B\u303C\u3041-\u3096\u309D-\u309F\u30A1-\u30FA\u30FC-\u30FF\u3105-\u312F\u3131-\u318E\u31A0-\u31BF\u31F0-\u31FF\u3400-\u4DBF\u4E00-\u9FFC\uA000-\uA48C\uA4D0-\uA4FD\uA500-\uA60C\uA610-\uA61F\uA62A\uA62B\uA640-\uA66E\uA67F-\uA69D\uA6A0-\uA6E5\uA717-\uA71F\uA722-\uA788\uA78B-\uA7BF\uA7C2-\uA7CA\uA7F5-\uA801\uA803-\uA805\uA807-\uA80A\uA80C-\uA822\uA840-\uA873\uA882-\uA8B3\uA8F2-\uA8F7\uA8FB\uA8FD\uA8FE\uA90A-\uA925\uA930-\uA946\uA960-\uA97C\uA984-\uA9B2\uA9CF\uA9E0-\uA9E4\uA9E6-\uA9EF\uA9FA-\uA9FE\uAA00-\uAA28\uAA40-\uAA42\uAA44-\uAA4B\uAA60-\uAA76\uAA7A\uAA7E-\uAAAF\uAAB1\uAAB5\uAAB6\uAAB9-\uAABD\uAAC0\uAAC2\uAADB-\uAADD\uAAE0-\uAAEA\uAAF2-\uAAF4\uAB01-\uAB06\uAB09-\uAB0E\uAB11-\uAB16\uAB20-\uAB26\uAB28-\uAB2E\uAB30-\uAB5A\uAB5C-\uAB69\uAB70-\uABE2\uAC00-\uD7A3\uD7B0-\uD7C6\uD7CB-\uD7FB\uF900-\uFA6D\uFA70-\uFAD9\uFB00-\uFB06\uFB13-\uFB17\uFB1D\uFB1F-\uFB28\uFB2A-\uFB36\uFB38-\uFB3C\uFB3E\uFB40\uFB41\uFB43\uFB44\uFB46-\uFBB1\uFBD3-\uFD3D\uFD50-\uFD8F\uFD92-\uFDC7\uFDF0-\uFDFB\uFE70-\uFE74\uFE76-\uFEFC\uFF21-\uFF3A\uFF41-\uFF5A\uFF66-\uFFBE\uFFC2-\uFFC7\uFFCA-\uFFCF\uFFD2-\uFFD7\uFFDA-\uFFDC\u10000-\u1000B\u1000D-\u10026\u10028-\u1003A\u1003C\u1003D\u1003F-\u1004D\u10050-\u1005D\u10080-\u100FA\u10280-\u1029C\u102A0-\u102D0\u10300-\u1031F\u1032D-\u10340\u10342-\u10349\u10350-\u10375\u10380-\u1039D\u103A0-\u103C3\u103C8-\u103CF\u10400-\u1049D\u104B0-\u104D3\u104D8-\u104FB\u10500-\u10527\u10530-\u10563\u10600-\u10736\u10740-\u10755\u10760-\u10767\u10800-\u10805\u10808\u1080A-\u10835\u10837\u10838\u1083C\u1083F-\u10855\u10860-\u10876\u10880-\u1089E\u108E0-\u108F2\u108F4\u108F5\u10900-\u10915\u10920-\u10939\u10980-\u109B7\u109BE\u109BF\u10A00\u10A10-\u10A13\u10A15-\u10A17\u10A19-\u10A35\u10A60-\u10A7C\u10A80-\u10A9C\u10AC0-\u10AC7\u10AC9-\u10AE4\u10B00-\u10B35\u10B40-\u10B55\u10B60-\u10B72\u10B80-\u10B91\u10C00-\u10C48\u10C80-\u10CB2\u10CC0-\u10CF2\u10D00-\u10D23\u10E80-\u10EA9\u10EB0\u10EB1\u10F00-\u10F1C\u10F27\u10F30-\u10F45\u10FB0-\u10FC4\u10FE0-\u10FF6\u11003-\u11037\u11083-\u110AF\u110D0-\u110E8\u11103-\u11126\u11144\u11147\u11150-\u11172\u11176\u11183-\u111B2\u111C1-\u111C4\u111DA\u111DC\u11200-\u11211\u11213-\u1122B\u11280-\u11286\u11288\u1128A-\u1128D\u1128F-\u1129D\u1129F-\u112A8\u112B0-\u112DE\u11305-\u1130C\u1130F\u11310\u11313-\u11328\u1132A-\u11330\u11332\u11333\u11335-\u11339\u1133D\u11350\u1135D-\u11361\u11400-\u11434\u11447-\u1144A\u1145F-\u11461\u11480-\u114AF\u114C4\u114C5\u114C7\u11580-\u115AE\u115D8-\u115DB\u11600-\u1162F\u11644\u11680-\u116AA\u116B8\u11700-\u1171A\u11800-\u1182B\u118A0-\u118DF\u118FF-\u11906\u11909\u1190C-\u11913\u11915\u11916\u11918-\u1192F\u1193F\u11941\u119A0-\u119A7\u119AA-\u119D0\u119E1\u119E3\u11A00\u11A0B-\u11A32\u11A3A\u11A50\u11A5C-\u11A89\u11A9D\u11AC0-\u11AF8\u11C00-\u11C08\u11C0A-\u11C2E\u11C40\u11C72-\u11C8F\u11D00-\u11D06\u11D08\u11D09\u11D0B-\u11D30\u11D46\u11D60-\u11D65\u11D67\u11D68\u11D6A-\u11D89\u11D98\u11EE0-\u11EF2\u11FB0\u12000-\u12399\u12480-\u12543\u13000-\u1342E\u14400-\u14646\u16800-\u16A38\u16A40-\u16A5E\u16AD0-\u16AED\u16B00-\u16B2F\u16B40-\u16B43\u16B63-\u16B77\u16B7D-\u16B8F\u16E40-\u16E7F\u16F00-\u16F4A\u16F50\u16F93-\u16F9F\u16FE0\u16FE1\u16FE3\u17000-\u187F7\u18800-\u18CD5\u18D00-\u18D08\u1B000-\u1B11E\u1B150-\u1B152\u1B164-\u1B167\u1B170-\u1B2FB\u1BC00-\u1BC6A\u1BC70-\u1BC7C\u1BC80-\u1BC88\u1BC90-\u1BC99\u1D400-\u1D454\u1D456-\u1D49C\u1D49E\u1D49F\u1D4A2\u1D4A5\u1D4A6\u1D4A9-\u1D4AC\u1D4AE-\u1D4B9\u1D4BB\u1D4BD-\u1D4C3\u1D4C5-\u1D505\u1D507-\u1D50A\u1D50D-\u1D514\u1D516-\u1D51C\u1D51E-\u1D539\u1D53B-\u1D53E\u1D540-\u1D544\u1D546\u1D54A-\u1D550\u1D552-\u1D6A5\u1D6A8-\u1D6C0\u1D6C2-\u1D6DA\u1D6DC-\u1D6FA\u1D6FC-\u1D714\u1D716-\u1D734\u1D736-\u1D74E\u1D750-\u1D76E\u1D770-\u1D788\u1D78A-\u1D7A8\u1D7AA-\u1D7C2\u1D7C4-\u1D7CB\u1E100-\u1E12C\u1E137-\u1E13D\u1E14E\u1E2C0-\u1E2EB\u1E800-\u1E8C4\u1E900-\u1E943\u1E94B\u1EE00-\u1EE03\u1EE05-\u1EE1F\u1EE21\u1EE22\u1EE24\u1EE27\u1EE29-\u1EE32\u1EE34-\u1EE37\u1EE39\u1EE3B\u1EE42\u1EE47\u1EE49\u1EE4B\u1EE4D-\u1EE4F\u1EE51\u1EE52\u1EE54\u1EE57\u1EE59\u1EE5B\u1EE5D\u1EE5F\u1EE61\u1EE62\u1EE64\u1EE67-\u1EE6A\u1EE6C-\u1EE72\u1EE74-\u1EE77\u1EE79-\u1EE7C\u1EE7E\u1EE80-\u1EE89\u1EE8B-\u1EE9B\u1EEA1-\u1EEA3\u1EEA5-\u1EEA9\u1EEAB-\u1EEBB\u20000-\u2A6DD\u2A700-\u2B734\u2B740-\u2B81D\u2B820-\u2CEA1\u2CEB0-\u2EBE0\u2F800-\u2FA1D\u30000-\u3134A]";

impl CodepointGenerator {
    pub fn new() -> Self {
        Self {
            regex: rand_regex::Regex::compile(UNICODE_LETTERS, 100).unwrap(),
        }
    }
    pub fn random_unicode_letter(&self, mut rng: &mut ThreadRng) -> char {
        let mut string: String = (&mut rng).sample_iter(&self.regex).next().unwrap();
        string.pop().unwrap()
    }

    pub fn random_codepoint(&self) -> char {
        let mut rng = rand::thread_rng();
        loop {
            let codepoint = self.random_unicode_letter(&mut rng) as u32;

            //let codepoint: u32 = rng.gen_range(0x0021u32, 0xEFFFFu32);
            if !(codepoint & NONCHARACTER_MASK == NONCHARACTER_MASK
                || FORBIDDEN_CODEPOINTS.contains(&codepoint)
                || FORBIDDEN_RANGES[0].contains(&codepoint)
                || FORBIDDEN_RANGES[1].contains(&codepoint)
                || from_u32(codepoint).is_none()
                || codepoint > 0xEFFFFu32)
            {
                if let Some(character) = from_u32(codepoint) {
                    let name = unicode_names2::name(character)
                        .map(|name| name.to_string())
                        .unwrap_or("<invalid>".to_owned())
                        .to_lowercase();
                    if name.contains("cjk") || name.contains("<invalid>") {
                        if rng.gen_bool(0.9) {
                            println!("Reject CJK");
                            // Refuse CJK Codepoints (most of the time).
                            continue;
                        }
                    }
                    return character;
                }
            }
        }
    }
}

pub fn get_unicode_codepoints() -> Vec<(u32, String)> {
    let unicode_codepoints = include_str!("../resources/UnicodeData.txt");
    let codepoints = unicode_codepoints.lines();
    let mut named_codepoints: Vec<(u32, String)> = Vec::new();
    print!("[");
    for codepoint in codepoints {
        let mut tokens = codepoint.split(';');
        let mut range = tokens.next().unwrap().trim().split("..");
        let name = tokens.next().unwrap().trim();

        // Names starting with "<" are private use areas or control
        if name.starts_with("<") {
            continue;
        }
        let id = range.next().unwrap();
        let id = u32::from_str_radix(id, 16).unwrap();
        print!("{}, ", id);
        named_codepoints.push((id, name.to_owned()));
    }
    println!("]");
    return named_codepoints;
}

pub fn get_unicode_blocks() -> Vec<(Range<u64>, String)> {
    let unicode_blocks = include_str!("../resources/Blocks.txt");
    let blocks = unicode_blocks.lines();
    let mut named_blocks: Vec<(Range<u64>, String)> = Vec::new();

    for block in blocks {
        if !(block.starts_with("#") || block.trim() == "") {
            let mut tokens = block.split(';');
            let mut range = tokens.next().unwrap().trim().split("..");
            let name = tokens.next().unwrap().trim();

            let from = range.next().unwrap();
            let to = range.next().unwrap();

            let from = u64::from_str_radix(from, 16).unwrap();
            let to = u64::from_str_radix(to, 16).unwrap();
            named_blocks.push(((from..to), name.to_owned()));
        }
    }
    return named_blocks;
}

pub fn count_unicode_chars() {
    let mut count = 0;

    for table in unicode_table::BY_NAME {
        println!("{}", table.0);
        count += table.1.len();
    }
    println!("{}", count);
}
