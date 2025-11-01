use lazy_static::lazy_static;

lazy_static! {
    pub(crate) static ref COLORS_MAP_VEC: Vec<(&'static str, (u8, u8, u8))> = {
        let mut v: Vec<(&'static str, (u8, u8, u8))> = vec![];
        // v.push(("Transparent", (0, 0, 0)));
        // ---v--- Free ---v---
        v.push(("Black", (0, 0, 0))); // id=1
        v.push(("Dark Gray", (60, 60, 60)));
        v.push(("Gray", (120, 120, 120)));
        v.push(("Light Gray", (210, 210, 210)));
        v.push(("White", (255, 255, 255)));
        v.push(("Deep Red", (96, 0, 24)));
        v.push(("Red", (237, 28, 36)));
        v.push(("Orange", (255, 127, 39)));
        v.push(("Gold", (246, 170, 9)));
        v.push(("Yellow", (249, 221, 59)));
        v.push(("Light Yellow", (255, 250, 188)));
        v.push(("Dark Green", (14, 185, 104)));
        v.push(("Green", (19, 230, 123)));
        v.push(("Light Green", (135, 255, 94)));
        v.push(("Dark Teal", (12, 129, 110)));
        v.push(("Teal", (16, 174, 166)));
        v.push(("Light Teal", (19, 225, 190)));
        v.push(("Dark Blue", (40, 80, 158)));
        v.push(("Blue", (64, 147, 228)));
        v.push(("Cyan", (96, 247, 242)));
        v.push(("Indigo", (107, 80, 246)));
        v.push(("Light Indigo", (153, 177, 251)));
        v.push(("Dark Purple", (120, 12, 153)));
        v.push(("Purple", (170, 56, 185)));
        v.push(("Light Purple", (224, 159, 249)));
        v.push(("Dark Pink", (203, 0, 122)));
        v.push(("Pink", (236, 31, 128)));
        v.push(("Light Pink", (243, 141, 169)));
        v.push(("Dark Brown", (104, 70, 52)));
        v.push(("Brown", (149, 104, 42)));
        v.push(("Beige", (248, 178, 119))); // id=31
        // ---^--- Free ---^---
        // ---v--- Paid ---v---
        v.push(("Medium Gray", (170, 170, 170))); // id=32
        v.push(("Dark Red", (165, 14, 30)));
        v.push(("Light Red", (250, 128, 114)));
        v.push(("Dark Orange", (228, 92, 26)));
        v.push(("Light Tan", (214, 181, 148)));
        v.push(("Dark Goldenrod", (156, 132, 49)));
        v.push(("Goldenrod", (197, 173, 49)));
        v.push(("Light Goldenrod", (232, 212, 95)));
        v.push(("Dark Olive", (74, 107, 58)));
        v.push(("Olive", (90, 148, 74)));
        v.push(("Light Olive", (132, 197, 115)));
        v.push(("Dark Cyan", (15, 121, 159)));
        v.push(("Light Cyan", (187, 250, 242)));
        v.push(("Light Blue", (125, 199, 255)));
        v.push(("Dark Indigo", (77, 49, 184)));
        v.push(("Dark Slate Blue", (74, 66, 132)));
        v.push(("Slate Blue", (122, 113, 196)));
        v.push(("Light Slate Blue", (181, 174, 241)));
        v.push(("Light Brown", (219, 164, 99)));
        v.push(("Dark Beige", (209, 128, 81)));
        v.push(("Light Beige", (255, 197, 165)));
        v.push(("Dark Peach", (155, 82, 73)));
        v.push(("Peach", (209, 128, 120)));
        v.push(("Light Peach", (250, 182, 164)));
        v.push(("Dark Tan", (123, 99, 82)));
        v.push(("Tan", (156, 132, 107)));
        v.push(("Dark Slate", (51, 57, 65)));
        v.push(("Slate", (109, 117, 141)));
        v.push(("Light Slate", (179, 185, 209)));
        v.push(("Dark Stone", (109, 100, 63)));
        v.push(("Stone", (148, 140, 107)));
        v.push(("Light Stone", (205, 197, 158))); // id=63
        // ---^--- Paid ---^---
        v
    };

    pub(crate) static ref COLORS_MAP: std::collections::HashMap<&'static str, (u8, u8, u8)> = {
        COLORS_MAP_VEC
            .iter()
            .cloned()
            .collect::<std::collections::HashMap<&'static str, (u8, u8, u8)>>()
    };
}

fn color_distance(c1: (u8, u8, u8), c2: (u8, u8, u8)) -> u32 {
    (c1.0 as i32 - c2.0 as i32).pow(2) as u32
        + (c1.1 as i32 - c2.1 as i32).pow(2) as u32
        + (c1.2 as i32 - c2.2 as i32).pow(2) as u32
}

pub(crate) fn find_color_name(pixel: &image::Rgba<u8>) -> &'static str {
    if pixel[3] == 0 {
        return "Transparent";
    }

    let rgb = (pixel[0], pixel[1], pixel[2]);
    if let Some((name, _)) = COLORS_MAP.iter().find(|(_, value)| **value == rgb) {
        return name;
    }

    // not found, find the closest one
    let mut closest_name = "";
    let mut closest_distance = u32::MAX;
    for (name, &value) in COLORS_MAP.iter() {
        let dist = color_distance(rgb, value);
        if dist < closest_distance {
            closest_distance = dist;
            closest_name = name;
        }
    }
    closest_name
}
