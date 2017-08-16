
use nom::*;

// JFIF
#[derive(Debug, PartialEq)]
enum Units {
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
struct JfifHeader<'a> {
    maj_ver: u8,
    min_ver: u8,
    units: Units,
    hor_dens: u16,
    ver_dens: u16,
    thumb_width: u8,
    thumb_height: u8,
    thumbnail: &'a [u8],
}

// TODO verify thumbnail size with header size
named!(parse_jfif<JfifHeader>,
    do_parse!(
        tag!(&[0xFF, 0xE0][..]) >> // APP0
        len: be_u16 >>
        tag!(&[0x4A, 0x46, 0x49, 0x46, 0x00][..]) >>
        maj: be_u8 >>
        min: be_u8 >>
        units: be_u8 >>
        xdens: be_u16 >>
        ydens: be_u16 >>
        xthumb: be_u8 >>
        ythumb: be_u8 >>
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

named!(parse_soi, tag!(&[0xFF, 0xD8]));
named!(parse_eoi, tag!(&[0xFF, 0xD9]));

named!(parse_comment,
    do_parse!(
        tag!(&[0xFF, 0xFE][..]) >>
        len: be_u16 >>
        com: take!(len) >>
        (com)
    )
);

named!(parse_appn,
    do_parse!(
        tag!(&[0xFF]) >>
        alt!(
            tag!(&[0xE0]) | tag!(&[0xE1]) | tag!(&[0xE2]) | tag!(&[0xE3]) |
            tag!(&[0xE4]) | tag!(&[0xE5]) | tag!(&[0xE6]) | tag!(&[0xE7]) |
            tag!(&[0xE8]) | tag!(&[0xE9]) | tag!(&[0xEA]) | tag!(&[0xEB]) |
            tag!(&[0xEC]) | tag!(&[0xED]) | tag!(&[0xEE]) | tag!(&[0xEF])
        ) >>
        len: be_u16 >>
        data: take!(len) >>
        (data)
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let img = include_bytes!("../test/test001.jpg");
        assert_eq!(parse_soi(&img[..]), IResult::Done(&img[2..], &[0xFF, 0xD8][..]));
        assert_eq!(parse_jfif(&img[2..]), IResult::Done(&img[20..],
            JfifHeader {
                maj_ver: 1,
                min_ver: 2,
                units:Units::Pixel,
                hor_dens: 1,
                ver_dens: 1,
                thumb_width: 0,
                thumb_height: 0,
                thumbnail: &[][..]
            }
        ));
        assert_eq!(parse_eoi(&img[20..]), IResult::Done(&[][..], &[0xFF, 0xD9][..]));
    }
}
