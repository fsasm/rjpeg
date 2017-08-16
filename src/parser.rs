
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
    units: Units,
    hor_dens: u16,
    ver_dens: u16,
    thumb_width: u8,
    thumb_height: u8,
    thumbnail: &'a[u8]
}

// TODO verify thumbnail size with header size
named!(jfif_header<JfifHeader>,
    do_parse!(
        tag!(&[0xFF, 0xE0][..]) >> // APP0
        len: be_u16 >>
        tag!(&[0x4A, 0x46, 0x49, 0x46, 0x00][..]) >>
        units: be_u8 >>
        xdens: be_u16 >>
        ydens: be_u16 >>
        xthumb: be_u8 >>
        ythumb: be_u8 >>
        thumb: take!(xthumb * ythumb * 3) >>
        (JfifHeader {
            units: Units::from_u8(units),
            hor_dens: xdens,
            ver_dens: ydens,
            thumb_width: xthumb,
            thumb_height: ythumb,
            thumbnail: thumb
        })
    )
);
