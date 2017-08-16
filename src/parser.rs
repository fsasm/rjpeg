
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
        ythumb: verify!(be_u8, |yt| (yt as u16 * xthumb as u16 * 3) + 16 == len) >>
        thumb: take!(xthumb * ythumb * 3) >>
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

pub enum TablesMisc<'a> {
    DRI(u16),
    APP(u8, &'a [u8]),
    COM(&'a [u8]),
    DHT,
    DQT,
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

named!(parse_tab_misc<TablesMisc>,
    alt!(parse_dri | parse_appn | parse_com)
);

named!(pub parse_jpeg<JfifHeader>,
    do_parse!(
        parse_soi >>
        jfif_header: parse_jfif >>
        many0!(parse_tab_misc) >>
        parse_eoi >>
        (jfif_header)
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
}
