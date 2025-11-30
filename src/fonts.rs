use dioxus::prelude::*;

const MINECRAFTER_ALT: Asset = asset!("/assets/fonts/Minecrafter/Minecrafter_Alt.ttf");
const MINECRAFTER_REG: Asset = asset!("/assets/fonts/Minecrafter/Minecrafter_Reg.ttf");

const MINECRAFTIA_REG: Asset = asset!("/assets/fonts/Minecraftia/Minecraftia_Regular.ttf");

const LILEX_REG: Asset = asset!("/assets/fonts/Lilex/Lilex-Regular.ttf");

#[component]
pub fn LoadFonts() -> Element {
    let minecrafter_alt_font_face = format!(
        r#"
        @font-face {{
            font-family: "MinecrafterAlt";
            src: url("{}") format("truetype");
            font-weight: normal;
            font-style: normal;
        }}
        "#,
        MINECRAFTER_ALT
    );

    let minecrafter_reg_font_face = format!(
        r#"
        @font-face {{
            font-family: "MinecrafterReg";
            src: url("{}") format("truetype");
            font-weight: normal;
            font-style: normal;
        }}
        "#,
        MINECRAFTER_REG
    );

    let minecraftia_reg_font_face = format!(
        r#"
        @font-face {{
            font-family: "MinecraftiaReg";
            src: url("{}") format("truetype");
            font-weight: normal;
            font-style: normal;
        }}
        "#,
        MINECRAFTIA_REG
    );

    let lilex_reg_font_face = format!(
        r#"
        @font-face {{
            font-family: "Lilex";
            src: url("{}") format("truetype");
            font-weight: normal;
            font-style: normal;
        }}
        "#,
        LILEX_REG
    );

    rsx! {
        style { "{minecrafter_alt_font_face}" }
        style { "{minecrafter_reg_font_face}" }
        style { "{minecraftia_reg_font_face}" }
        style { "{lilex_reg_font_face}" }
    }
}