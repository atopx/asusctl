use std::ops::Range;

use crate::ctrl_anime::device::{PACKET_LEN, AniMePacket};
use rog_types::anime_matrix::AniMePane;

// Starting from row 1
const PANE_ROWS : usize = 55;

type IndexRangeOption = Option<Range<usize>>;

pub struct AniMeParser;
impl AniMeParser {
    pub fn packets_from_pane(pane : AniMePane) -> (AniMePacket, AniMePacket) {
        let mut packet1 = [0u8; PACKET_LEN];
        let mut packet2 = [0u8; PACKET_LEN];
        for row in 1..=PANE_ROWS {
            let (pane_indexes_opt, packet1_indexes_opt, packet2_indexes_opt) =
                Self::indexes_from_row(row);

            if let Some(pane_indexes) = pane_indexes_opt {
                let pane_slice = &pane[pane_indexes];
                if let Some(indexes) = packet1_indexes_opt {
                    packet1[indexes].copy_from_slice(pane_slice);
                }
                if let Some(indexes) = packet2_indexes_opt {
                    packet2[indexes].copy_from_slice(pane_slice);
                }
            }
        }

        (packet1, packet2)
    }

    // Get an array of indexes for (pane, packet1, packet2) from row
    // All missing index are "holes": useless bytes that should keep value 0u8
    // Long matching for better understanding
    const fn indexes_from_row(row : usize) ->
        (IndexRangeOption, IndexRangeOption, IndexRangeOption) {
        match row {
            1  => (Some(0..0+32),       Some(8..8+32),     None),
            2  => (Some(32..32+33),     Some(41..41+33),   None),
            3  => (Some(65..65+33),     Some(76..76+33),   None),
            4  => (Some(98..98+33),     Some(109..109+33), None),
            5  => (Some(131..131+33),   Some(144..144+33), None),
            6  => (Some(164..164+33),   Some(177..177+33), None),
            7  => (Some(197..197+33),   Some(211..211+33), None),
            8  => (Some(230..230+32),   Some(244..244+32), None),
            9  => (Some(262..261+32),   Some(277..277+32), None),
            10 => (Some(294..294+31),   Some(309..309+31), None),
            11 => (Some(325..325+31),   Some(341..341+31), None),
            12 => (Some(356..356+30),   Some(372..372+30), None),
            13 => (Some(386..386+30),   Some(403..403+30), None),
            14 => (Some(416..416+29),   Some(433..433+29), None),
            15 => (Some(445..445+29),   Some(463..463+29), None),
            16 => (Some(474..473+28),   Some(492..492+28), None),
            17 => (Some(502..502+28),   Some(521..521+28), None),
            18 => (Some(530..530+27),   Some(549..549+27), None),
            19 => (Some(557..557+27),   Some(577..577+27), None),
            20 => (Some(584..584+26),   Some(604..604+26), None),
            21 => (Some(610..610+26),   Some(631..631+3),  Some(7..7+23)),
            22 => (Some(636..636+25),   None,              Some(30..30+25)),
            23 => (Some(661..661+25),   None,              Some(56..56+25)),
            24 => (Some(686..686+24),   None,              Some(81..81+24)),
            25 => (Some(710..710+24),   None,              Some(106..106+24)),
            26 => (Some(734..734+23),   None,              Some(130..130+23)),
            27 => (Some(758..758+23),   None,              Some(154..154+23)),
            28 => (Some(780..780+22),   None,              Some(177..177+22)),
            29 => (Some(802..802+22),   None,              Some(200..200+22)),
            30 => (Some(824..824+21),   None,              Some(222..222+21)),
            31 => (Some(845..845+21),   None,              Some(244..244+21)),
            32 => (Some(866..866+20),   None,              Some(265..284+20)),
            33 => (Some(886..886+20),   None,              Some(286..286+20)),
            34 => (Some(906..906+19),   None,              Some(306..306+19)),
            35 => (Some(925..925+19),   None,              Some(326..344+19)),
            36 => (Some(944..944+18),   None,              Some(345..345+18)),
            37 => (Some(962..962+18),   None,              Some(364..364+18)),
            38 => (Some(980..980+17),   None,              Some(382..382+17)),
            39 => (Some(997..997+17),   None,              Some(400..400+17)),
            40 => (Some(1014..1014+16), None,              Some(417..417+16)),
            41 => (Some(1030..1030+16), None,              Some(434..434+16)),
            42 => (Some(1046..1046+15), None,              Some(450..450+15)),
            43 => (Some(1061..1061+15), None,              Some(466..466+15)),
            44 => (Some(1076..1076+14), None,              Some(481..481+14)),
            45 => (Some(1090..1090+14), None,              Some(496..496+14)),
            46 => (Some(1104..1104+13), None,              Some(510..510+13)),
            47 => (Some(1117..1117+13), None,              Some(524..524+13)),
            48 => (Some(1130..1130+12), None,              Some(537..537+12)),
            49 => (Some(1142..1142+12), None,              Some(550..550+12)),
            50 => (Some(1154..1154+11), None,              Some(562..562+11)),
            51 => (Some(1165..1165+11), None,              Some(574..574+11)),
            52 => (Some(1176..1176+10), None,              Some(585..585+10)),
            53 => (Some(1186..1186+10), None,              Some(596..596+10)),
            54 => (Some(1196..1196+9),  None,              Some(606..606+9)),
            55 => (Some(1205..1205+9),  None,              Some(616..616+9)),
            _ => (None, None, None),
        }
    }
}
