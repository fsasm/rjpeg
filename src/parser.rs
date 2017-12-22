
use nom::*;

// JFIF
#[derive(Debug, PartialEq)]
pub enum Units {
    Pixel,
    DotsPerInch,
    DotsPerCm,
    Unknown,
}

impl Units {
    fn from_u8(units: u8) -> Self {
        match units {
            0 => Units::Pixel,
            1 => Units::DotsPerInch,
            2 => Units::DotsPerCm,
            _ => Units::Unknown,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct JfifHeader<'a> {
    maj_ver: u8,
    min_ver: u8,
    units: Units,
    hor_dens: u16,
    ver_dens: u16,
    thumb_width: u8,
    thumb_height: u8,
    thumbnail: &'a [u8],
}

named!(parse_jfif<JfifHeader>,
    do_parse!(
        tag!(&[0xFF, 0xE0]) >> // APP0
        len: be_u16 >>
        tag!(&[0x4A, 0x46, 0x49, 0x46, 0x00]) >>
        maj: be_u8 >>
        min: be_u8 >>
        units: be_u8 >>
        xdens: be_u16 >>
        ydens: be_u16 >>
        xthumb: be_u8 >>
        ythumb: be_u8 >>
        thumb_size: verify!(value!(xthumb as u16 * ythumb as u16 * 3), |val| val + 16 == len) >>
        thumb: take!(thumb_size) >>
        (JfifHeader {
            maj_ver: maj,
            min_ver: min,
            units: Units::from_u8(units),
            hor_dens: xdens,
            ver_dens: ydens,
            thumb_width: xthumb,
            thumb_height: ythumb,
            thumbnail: thumb
        })
    )
);

pub struct HuffTable<'a> {
    class: u8,
    dest: u8,
    cnt_len: [u8; 16],
    huff_val: &'a [u8]
}

#[derive(Debug, PartialEq)]
pub enum TablesMisc<'a> {
    DRI(u16),
    APP(u8, &'a [u8]),
    COM(&'a [u8]),
    DHT(&'a [u8]),
    DQT(&'a [u8]),
}

named!(parse_soi, tag!(&[0xFF, 0xD8]));
named!(parse_eoi, tag!(&[0xFF, 0xD9]));

named!(seg_len<u16>, verify!(be_u16, |val: u16| val >= 2));

named!(parse_com<TablesMisc>,
    do_parse!(
        tag!(&[0xFF, 0xFE]) >>
        len: seg_len >>
        com: take!(len - 2) >>
        (TablesMisc::COM(com))
    )
);

named!(parse_appn<TablesMisc>,
    do_parse!(
        tag!(&[0xFF]) >>
        n: alt!(
            tag!(&[0xE0]) | tag!(&[0xE1]) | tag!(&[0xE2]) | tag!(&[0xE3]) |
            tag!(&[0xE4]) | tag!(&[0xE5]) | tag!(&[0xE6]) | tag!(&[0xE7]) |
            tag!(&[0xE8]) | tag!(&[0xE9]) | tag!(&[0xEA]) | tag!(&[0xEB]) |
            tag!(&[0xEC]) | tag!(&[0xED]) | tag!(&[0xEE]) | tag!(&[0xEF])
        ) >>
        len: seg_len >>
        data: take!(len - 2) >>
        (TablesMisc::APP(n[0] - 0xE0, data))
    )
);

named!(parse_dnl<u16>,
    do_parse!(
        tag!(&[0xFF, 0xDC]) >>
        tag!(&[0x00, 0x04]) >> // length is fixed to 4
        dnl: verify!(be_u16, |val: u16| val > 0) >>
        (dnl)
    )
);

named!(parse_dri<TablesMisc>,
    do_parse!(
        tag!(&[0xFF, 0xDD]) >>
        tag!(&[0x00, 0x04]) >> // length is fixed to 4
        dri: be_u16 >>
        (TablesMisc::DRI(dri))
    )
);

fn sum(arr: [u8; 16]) -> u16 {
    arr.iter().fold(0u16, |acc, x| acc + (*x) as u16)
}

named!(parse_huffhead<(u8, u8, [u8; 16], u16)>,
    do_parse!(
        cd: verify!(be_u8, |val| (val >> 4) < 2u8 && (val & 0x0F) < 4u8) >>
        count_len: count_fixed!(u8, be_u8, 16) >>
        sum: verify!(value!(sum(count_len)), |val| val <= 256) >>
        (cd >> 4, cd & 0x0F, count_len, sum)
    )
);

named!(parse_hufftable<HuffTable>,
    do_parse!(
        head: parse_huffhead >>
        huffvals: take!(head.3) >>
        (HuffTable {
            class: head.0,
            dest: head.1,
            cnt_len: head.2,
            huff_val: huffvals
        })
    )
);

named!(parse_dht<TablesMisc>,
    do_parse!(
        tag!(&[0xFF, 0xC4]) >>
        len: seg_len >>
        body: take!(len - 2) >>
        (TablesMisc::DHT(body))
    )
);

/* TODO like for DHT this is just a dummy parser */
named!(parse_dqt<TablesMisc>,
    do_parse!(
        tag!(&[0xFF, 0xDB]) >>
        len: seg_len >>
        body: take!(len - 2) >>
        (TablesMisc::DQT(body))
    )
);

/* FIXME also parse the huffman tables. Problem is that the number of
 * tables depends on the number of symbols in a table.
named!(parse_dht<Vec<HuffTable> >,
    do_parse!(
        tag!(&[0xFF, 0xC4]) >>
        len: seg_len >>
        body: take!(len - 2) >>
        (fold_many0!(body, parse_hufftable, Vec::new(), |mut v: Vec<_>, elem| {
            v.push(elem);
            v
        }).unwrap().1)
    )
);
*/

pub struct CompParam {
    id: u8,
    hor_sampling: u8,
    ver_sampling: u8,
    quant_selector: u8,
}

pub struct FrameHeader{
    precision: u8,
    height: u16,
    width: u16,
    comp_params: Vec<CompParam>
}

named!(parse_comp_param<CompParam>,
    do_parse!(
        id: be_u8 >>
        sampling: be_u8 >>
        selector: be_u8 >>
        (CompParam {
            id: id,
            hor_sampling: sampling >> 4,
            ver_sampling: sampling & 0x0F,
            quant_selector: selector
        })
    )
);

named!(parse_sof0<FrameHeader>,
    do_parse!(
        tag!(&[0xFF, 0xC0]) >>
        len: seg_len >>
        prec: be_u8 >>
        height: be_u16 >>
        width: be_u16 >>
        num_comp: be_u8 >>
        comp_params: many_m_n!(num_comp as usize, num_comp as usize, parse_comp_param) >>
        (FrameHeader {
            precision: prec,
            height: height,
            width: width,
            comp_params: comp_params
        })
    )
);

#[derive(Debug, PartialEq)]
pub struct Jpeg<'a> {
    jfif_header: JfifHeader<'a>,
    tables: Vec<TablesMisc<'a>>
}


named!(parse_tab_misc<TablesMisc>,
    alt!(parse_dri | parse_appn | parse_com | parse_dht | parse_dqt)
);

named!(pub parse_jpeg<Jpeg>,
    do_parse!(
        parse_soi >>
        jfif_header: parse_jfif >>
        tables: many0!(parse_tab_misc) >>
        opt!(parse_sof0) >>
        parse_eoi >>
        (Jpeg {
            jfif_header: jfif_header,
            tables: tables
        })
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let img = include_bytes!("../test/test001.jpg");
        assert!(parse_jpeg(img).is_done());
    }

    #[test]
    fn ign_appn() {
        let img = include_bytes!("../test/test002.jpg");
        assert!(parse_jpeg(img).is_done());
    }

    #[test]
    fn comment() {
        let img = include_bytes!("../test/test003.jpg");
        assert!(parse_jpeg(img).is_done());
    }

    #[test]
    fn broken_jfif() {
        let img = include_bytes!("../test/test004.jpg");
        assert!(parse_jpeg(img).is_err());
    }

    #[test]
    fn empty_dht_dqt() {
        let img = include_bytes!("../test/test005.jpg");
        assert!(parse_jpeg(img).is_done());
    }
}
