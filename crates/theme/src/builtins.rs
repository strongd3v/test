use std::collections::BTreeMap;
use std::sync::OnceLock;

use sha2::{Digest, Sha256};

use crate::{
    AccentRoles, Appearance, Color, SurfaceTreatment, TerminalPalette, ThemeColors, ThemeFamily,
    ThemeRegistry, ThemeSource, ThemeVariant,
};

pub fn builtin_registry() -> &'static ThemeRegistry {
    static REGISTRY: OnceLock<ThemeRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| ThemeRegistry {
        families: vec![
            family("space", "Space", vec![space_light(), space_dark()]),
            family("ruby", "Ruby", vec![ruby_light(), ruby_dark()]),
            family("wine", "Wine", vec![wine_light(), wine_dark()]),
            family(
                "terracotta",
                "Terracotta",
                vec![terracotta_light(), terracotta_dark()],
            ),
            family("sunset", "Sunset", vec![sunset_light(), sunset_dark()]),
            family("saffron", "Saffron", vec![saffron_light(), saffron_dark()]),
            family("lime", "Lime", vec![lime_light(), lime_dark()]),
            family("forest", "Forest", vec![forest_light(), forest_dark()]),
            family("jade", "Jade", vec![jade_light(), jade_dark()]),
            family("lagoon", "Lagoon", vec![lagoon_light(), lagoon_dark()]),
            family("aqua", "Aqua", vec![aqua_light(), aqua_dark()]),
            family("cobalt", "Cobalt", vec![cobalt_light(), cobalt_dark()]),
            family(
                "midnight",
                "Midnight",
                vec![midnight_light(), midnight_dark()],
            ),
            family(
                "indigo",
                "Indigo",
                vec![indigo_light(), indigo_dark()],
            ),
            family("violet", "Violet", vec![violet_light(), violet_dark()]),
            family(
                "amethyst",
                "Amethyst",
                vec![amethyst_light(), amethyst_dark()],
            ),
            family(
                "magenta",
                "Magenta",
                vec![magenta_light(), magenta_dark()],
            ),
            family(
                "sakura",
                "Sakura",
                vec![sakura_light(), sakura_dark()],
            ),
            family(
                "silver",
                "Silver",
                vec![silver_light(), silver_dark()],
            ),
            family("cocoa", "Cocoa", vec![cocoa_light(), cocoa_dark()]),
        ],
    })
}

fn family(id: &str, name: &str, variants: Vec<ThemeVariant>) -> ThemeFamily {
    ThemeFamily {
        id: id.into(),
        name: name.into(),
        variants,
    }
}

struct Seeds<'a> {
    id: &'a str,
    family_id: &'a str,
    name: &'a str,
    appearance: Appearance,
    treatment: SurfaceTreatment,
    background: &'a str,
    shell: &'a str,
    raised: &'a str,
    card: &'a str,
    text: &'a str,
    muted: &'a str,
    faint: &'a str,
    accent: &'a str,
    danger: &'a str,
    warning: &'a str,
    success: &'a str,
    terminal_background: &'a str,
    ansi: [&'a str; 16],
    syntax: [&'a str; 12],
    source: ThemeSource,
}

fn variant(seed: Seeds<'_>) -> ThemeVariant {
    let background = c(seed.background);
    let shell = c(seed.shell);
    let raised = c(seed.raised);
    let card = c(seed.card);
    let text = c(seed.text);
    let muted = c(seed.muted).ensure_contrast(background, 4.5);
    let faint = c(seed.faint);
    let danger = c(seed.danger);
    let warning = c(seed.warning);
    let success = c(seed.success);
    let accent = AccentRoles::derive(c(seed.accent), seed.appearance, background);
    let dark = seed.appearance.is_dark();
    let border_tone = if dark { Color::WHITE } else { Color::BLACK };
    let solid = if dark {
        Color::rgb(235, 235, 239)
    } else {
        Color::rgb(35, 35, 40)
    };
    let colors = ThemeColors {
        background,
        shell,
        raised,
        card,
        dialog: card.mix(raised, if dark { 0.18 } else { 0.04 }),
        overlay: card.mix(raised, if dark { 0.34 } else { 0.02 }),
        hover: border_tone.with_alpha(if dark { 0.11 } else { 0.06 }),
        active: accent.primary.with_alpha(if dark { 0.18 } else { 0.10 }),
        border: border_tone.with_alpha(if dark { 0.10 } else { 0.12 }),
        border_strong: border_tone.with_alpha(if dark { 0.18 } else { 0.22 }),
        text,
        text_muted: muted,
        text_faint: faint,
        solid,
        on_solid: solid.best_on_color(),
        danger,
        danger_muted: danger.mix(text, 0.28),
        warning,
        warning_muted: warning.mix(text, 0.25),
        success,
        success_muted: success.mix(text, 0.25),
        input: if dark { raised.with_alpha(0.72) } else { card },
        cursor: text.with_alpha(if dark { 0.40 } else { 0.55 }),
        diff_add: success,
        diff_delete: danger,
        diff_hunk: accent.primary.with_alpha(if dark { 0.08 } else { 0.07 }),
    };
    let terminal_background = c(seed.terminal_background);
    let mut variant = ThemeVariant {
        id: seed.id.into(),
        family_id: seed.family_id.into(),
        name: seed.name.into(),
        appearance: seed.appearance,
        recommended_surface_treatment: seed.treatment,
        colors,
        accent,
        syntax: syntax(seed.syntax),
        terminal: TerminalPalette {
            background: terminal_background,
            foreground: text.ensure_contrast(terminal_background, 4.5),
            selection: border_tone.with_alpha(if dark { 0.22 } else { 0.16 }),
            ansi: seed.ansi.map(c),
        },
        source: seed.source,
    };
    // Hash the checked-in resolved definition itself (with the hash field
    // blanked), not merely its source URL. This makes provenance sensitive to
    // curation edits as well as upstream revision changes.
    variant.source.asset_hash.clear();
    let encoded = serde_json::to_vec(&variant).expect("built-in theme serializes");
    variant.source.asset_hash = format!("sha256:{:x}", Sha256::digest(encoded));
    variant
}

fn syntax(colors: [&str; 12]) -> BTreeMap<String, Color> {
    let [
        comment,
        keyword,
        string,
        number,
        type_name,
        function,
        property,
        variable,
        punctuation,
        tag,
        attribute,
        invalid,
    ] = colors.map(c);
    BTreeMap::from([
        ("comment".into(), comment),
        ("keyword".into(), keyword),
        ("string".into(), string),
        ("stringSpecial".into(), attribute),
        ("escape".into(), attribute),
        ("number".into(), number),
        ("boolean".into(), number),
        ("type".into(), type_name),
        ("typeBuiltin".into(), type_name),
        ("constructor".into(), type_name),
        ("function".into(), function),
        ("functionBuiltin".into(), function),
        ("macro".into(), keyword),
        ("property".into(), property),
        ("constant".into(), number),
        ("variable".into(), variable),
        ("variableSpecial".into(), keyword),
        ("parameter".into(), variable),
        ("operator".into(), keyword),
        ("punctuation".into(), punctuation),
        ("tag".into(), tag),
        ("attribute".into(), attribute),
        ("label".into(), function),
        ("embedded".into(), punctuation),
        ("invalid".into(), invalid),
    ])
}

fn source(id: &str, format: &str, url: &str, revision: &str, license: &str) -> ThemeSource {
    ThemeSource {
        format: format.into(),
        url: url.into(),
        revision: revision.into(),
        license: license.into(),
        asset_hash: format!("pending:{id}"),
    }
}

fn c(value: &str) -> Color {
    value.parse().expect("built-in theme colors are valid")
}

const ANSI_DARK: [&str; 16] = [
    "#242424", "#f87171", "#4ade80", "#facc15", "#60a5fa", "#c084fc", "#22d3ee", "#d4d4d8",
    "#52525b", "#fca5a5", "#86efac", "#fde047", "#93c5fd", "#d8b4fe", "#67e8f9", "#fafafa",
];

const ANSI_LIGHT: [&str; 16] = [
    "#1f1f1f", "#dc2626", "#16a34a", "#b45309", "#2563eb", "#9333ea", "#0e7490", "#3f3f46",
    "#71717a", "#b91c1c", "#15803d", "#92400e", "#1d4ed8", "#7e22ce", "#155e75", "#18181b",
];

/// Shared text, semantic, and surface base colors. Every family keeps the same
/// neutral workbench skeleton; the identity lives in backgrounds, accents, and
/// secondary syntax tones.
const TEXT_DARK: &str = "#E8E8EA";
const MUTED_DARK: &str = "#A9A9AE";
const FAINT_DARK: &str = "#85858A";
const DANGER_DARK: &str = "#F87171";
const WARNING_DARK: &str = "#FACC15";
const SUCCESS_DARK: &str = "#4ADE80";
const TEXT_LIGHT: &str = "#303035";
const MUTED_LIGHT: &str = "#62626A";
const FAINT_LIGHT: &str = "#797981";
const DANGER_LIGHT: &str = "#DC2626";
const WARNING_LIGHT: &str = "#A16207";
const SUCCESS_LIGHT: &str = "#15803D";

const NATIVE_URL: &str = "https://github.com/strongd3v/test";
const NATIVE_REVISION: &str = "d138049";

/// One curated color identity shared by the family's light and dark variants.
struct Persona {
    slug: &'static str,
    name: &'static str,
    band: Band,
}

#[derive(Clone, Copy)]
enum Band {
    /// Standard vivid accomplice set: green strings, gold numbers, blue
    /// functions, pink properties, cyan attributes.
    Neutral,
    /// Green-family accents swap strings to gold and numbers to orange so the
    /// keyword stays the only dominant green on screen.
    Green,
    /// Blue-family accents swap functions to pink and properties to violet.
    Blue,
    /// Pink-family accents swap properties to violet.
    Pink,
}

fn syntax_colors(
    band: Band,
    accent: &'static str,
    secondary: &'static str,
    dark: bool,
) -> [&'static str; 12] {
    const COMMENT_DARK: &str = "#92929A";
    const PUNCT_DARK: &str = "#A1A1AA";
    const COMMENT_LIGHT: &str = "#6B7280";
    const PUNCT_LIGHT: &str = "#52525B";
    if dark {
        match band {
            Band::Neutral => [
                COMMENT_DARK, accent, "#34D399", "#FACC15", secondary, "#60A5FA", "#F472B6",
                TEXT_DARK, PUNCT_DARK, secondary, "#22D3EE", DANGER_DARK,
            ],
            Band::Green => [
                COMMENT_DARK, accent, "#FACC15", "#F97316", secondary, "#60A5FA", "#C084FC",
                TEXT_DARK, PUNCT_DARK, secondary, "#F472B6", DANGER_DARK,
            ],
            Band::Blue => [
                COMMENT_DARK, accent, "#34D399", "#FACC15", secondary, "#F472B6", "#C084FC",
                TEXT_DARK, PUNCT_DARK, secondary, "#22D3EE", DANGER_DARK,
            ],
            Band::Pink => [
                COMMENT_DARK, accent, "#34D399", "#FACC15", secondary, "#60A5FA", "#C084FC",
                TEXT_DARK, PUNCT_DARK, secondary, "#22D3EE", DANGER_DARK,
            ],
        }
    } else {
        match band {
            Band::Neutral => [
                COMMENT_LIGHT, accent, "#15803D", "#A16207", secondary, "#2563EB", "#BE185D",
                TEXT_LIGHT, PUNCT_LIGHT, secondary, "#0E7490", DANGER_LIGHT,
            ],
            Band::Green => [
                COMMENT_LIGHT, accent, "#A16207", "#C2410C", secondary, "#1D4ED8", "#7E22CE",
                TEXT_LIGHT, PUNCT_LIGHT, secondary, "#BE185D", DANGER_LIGHT,
            ],
            Band::Blue => [
                COMMENT_LIGHT, accent, "#15803D", "#A16207", secondary, "#BE185D", "#7E22CE",
                TEXT_LIGHT, PUNCT_LIGHT, secondary, "#0E7490", DANGER_LIGHT,
            ],
            Band::Pink => [
                COMMENT_LIGHT, accent, "#15803D", "#A16207", secondary, "#1D4ED8", "#7E22CE",
                TEXT_LIGHT, PUNCT_LIGHT, secondary, "#0E7490", DANGER_LIGHT,
            ],
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn half(
    persona: &Persona,
    appearance: Appearance,
    background: &'static str,
    shell: &'static str,
    raised: &'static str,
    card: &'static str,
    accent: &'static str,
    secondary: &'static str,
    terminal_background: &'static str,
) -> ThemeVariant {
    let slug = persona.slug;
    let dark = appearance.is_dark();
    // Saffron's gold accent sits next to the shared amber warning, so the
    // family shifts its warning one step toward orange to keep every role a
    // distinct tone.
    let warning =
        if dark {
            if slug == "saffron" { "#F97316" } else { WARNING_DARK }
        } else {
            if slug == "saffron" { "#C2570D" } else { WARNING_LIGHT }
        };
    variant(Seeds {
        id: if dark {
            Box::leak(format!("{slug}-dark").into_boxed_str())
        } else {
            Box::leak(format!("{slug}-light").into_boxed_str())
        },
        family_id: slug,
        name: persona.name,
        appearance,
        treatment: SurfaceTreatment::Frosted,
        background,
        shell,
        raised,
        card,
        text: if dark { TEXT_DARK } else { TEXT_LIGHT },
        muted: if dark { MUTED_DARK } else { MUTED_LIGHT },
        faint: if dark { FAINT_DARK } else { FAINT_LIGHT },
        accent,
        danger: if dark { DANGER_DARK } else { DANGER_LIGHT },
        warning,
        success: if dark { SUCCESS_DARK } else { SUCCESS_LIGHT },
        terminal_background,
        ansi: if dark { ANSI_DARK } else { ANSI_LIGHT },
        syntax: syntax_colors(persona.band, accent, secondary, dark),
        source: source(slug, "native", NATIVE_URL, NATIVE_REVISION, "MIT"),
    })
}

fn space_dark() -> ThemeVariant {
    variant(Seeds {
        id: "space-dark",
        family_id: "space",
        name: "Space Dark",
        appearance: Appearance::Dark,
        treatment: SurfaceTreatment::Frosted,
        background: "#060606",
        shell: "#0d0d0d",
        raised: "#343438",
        card: "#0e0e0e",
        text: "#e8e8ea",
        muted: "#a9a9ae",
        faint: "#85858a",
        accent: "#0091ff",
        danger: "#f87171",
        warning: "#facc15",
        success: "#34d399",
        terminal_background: "#090909",
        ansi: ANSI_DARK,
        syntax: [
            "#92929a", "#0091ff", "#34d399", "#facc15", "#c084fc", "#60a5fa", "#f472b6", "#e8e8ea",
            "#a1a1aa", "#c084fc", "#22d3ee", "#f87171",
        ],
        source: source(
            "space-dark",
            "native",
            NATIVE_URL,
            NATIVE_REVISION,
            "MIT",
        ),
    })
}

fn space_light() -> ThemeVariant {
    variant(Seeds {
        id: "space-light",
        family_id: "space",
        name: "Space Light",
        appearance: Appearance::Light,
        treatment: SurfaceTreatment::Frosted,
        background: "#ffffff",
        shell: "#f3f3f5",
        raised: "#ededf0",
        card: "#ffffff",
        text: "#303035",
        muted: "#62626a",
        faint: "#797981",
        accent: "#0088ff",
        danger: "#dc2626",
        warning: "#a16207",
        success: "#15803d",
        terminal_background: "#fafafa",
        ansi: ANSI_LIGHT,
        syntax: [
            "#6b7280", "#0088ff", "#15803d", "#a16207", "#7e22ce", "#2563eb", "#be185d", "#303035",
            "#52525b", "#7e22ce", "#0e7490", "#b91c1c",
        ],
        source: source(
            "space-light",
            "native",
            NATIVE_URL,
            NATIVE_REVISION,
            "MIT",
        ),
    })
}

fn ruby_light() -> ThemeVariant {
    half(
        &Persona { slug: "ruby", name: "Ruby", band: Band::Neutral },
        Appearance::Light,
        "#FCF0F1", "#F3E4E6", "#E7D3D7", "#FFF9FA",
        "#C81E30", "#B93A4A", "#FBF2F2",
    )
}

fn ruby_dark() -> ThemeVariant {
    half(
        &Persona { slug: "ruby", name: "Ruby", band: Band::Neutral },
        Appearance::Dark,
        "#170A0D", "#211216", "#38232C", "#1C0F13",
        "#FF4655", "#FF92A2", "#100709",
    )
}

fn wine_light() -> ThemeVariant {
    half(
        &Persona { slug: "wine", name: "Wine", band: Band::Neutral },
        Appearance::Light,
        "#FAF0F3", "#F1E3E8", "#E4D2DA", "#FDF8FA",
        "#99203E", "#8C2A48", "#F9F1F4",
    )
}

fn wine_dark() -> ThemeVariant {
    half(
        &Persona { slug: "wine", name: "Wine", band: Band::Neutral },
        Appearance::Dark,
        "#160A10", "#201220", "#3A2534", "#1B0F17",
        "#D64864", "#E27A96", "#0F070C",
    )
}

fn terracotta_light() -> ThemeVariant {
    half(
        &Persona { slug: "terracotta", name: "Terracotta", band: Band::Neutral },
        Appearance::Light,
        "#FBF1EA", "#F2E7DD", "#E9D9C9", "#FEFAF5",
        "#B04A18", "#9A4A1E", "#FAF3EE",
    )
}

fn terracotta_dark() -> ThemeVariant {
    half(
        &Persona { slug: "terracotta", name: "Terracotta", band: Band::Neutral },
        Appearance::Dark,
        "#170D09", "#201810", "#3A2A1E", "#1C120C",
        "#E8703C", "#F19A6B", "#100A07",
    )
}

fn sunset_light() -> ThemeVariant {
    half(
        &Persona { slug: "sunset", name: "Sunset", band: Band::Neutral },
        Appearance::Light,
        "#FCF2E8", "#F3E7DB", "#EBDCCE", "#FFFBF4",
        "#C05C0A", "#A85A12", "#FBF5EE",
    )
}

fn sunset_dark() -> ThemeVariant {
    half(
        &Persona { slug: "sunset", name: "Sunset", band: Band::Neutral },
        Appearance::Dark,
        "#180D06", "#211610", "#3A2B1C", "#1C110A",
        "#FF8F3C", "#FFB96E", "#100B07",
    )
}

fn saffron_light() -> ThemeVariant {
    half(
        &SAFFRON,
        Appearance::Light,
        "#FBF5E5", "#F2ECDA", "#EAE3C8", "#FFFDF4",
        "#A57600", "#8F6A00", "#FAF7EB",
    )
}

fn saffron_dark() -> ThemeVariant {
    half(
        &SAFFRON,
        Appearance::Dark,
        "#181106", "#211A0E", "#3A331E", "#1C140C",
        "#FFCF4D", "#FFDF80", "#100E08",
    )
}

const SAFFRON: Persona = Persona { slug: "saffron", name: "Saffron", band: Band::Neutral };

fn lime_light() -> ThemeVariant {
    half(
        &Persona { slug: "lime", name: "Lime", band: Band::Green },
        Appearance::Light,
        "#F0F7E4", "#E6EDD8", "#DCE5C8", "#FAFEF0",
        "#638000", "#556A00", "#F2F8E8",
    )
}

fn lime_dark() -> ThemeVariant {
    half(
        &Persona { slug: "lime", name: "Lime", band: Band::Green },
        Appearance::Dark,
        "#11150A", "#1A1E12", "#2E3420", "#15180E",
        "#C7EE4F", "#D6F58B", "#0C1008",
    )
}

fn forest_light() -> ThemeVariant {
    half(
        &Persona { slug: "forest", name: "Forest", band: Band::Green },
        Appearance::Light,
        "#EBF5EC", "#E0EAE1", "#D5E3D6", "#F6FCF7",
        "#1D8040", "#2A6B44", "#F0F7F1",
    )
}

fn forest_dark() -> ThemeVariant {
    half(
        &Persona { slug: "forest", name: "Forest", band: Band::Green },
        Appearance::Dark,
        "#091309", "#121B12", "#223320", "#0F160F",
        "#3CC96C", "#7EDFA6", "#070E07",
    )
}

fn jade_light() -> ThemeVariant {
    half(
        &Persona { slug: "jade", name: "Jade", band: Band::Green },
        Appearance::Light,
        "#EAF6F1", "#DFEAE5", "#D2E3DA", "#F4FBF7",
        "#15845C", "#1F7A5C", "#EFF8F5",
    )
}

fn jade_dark() -> ThemeVariant {
    half(
        &Persona { slug: "jade", name: "Jade", band: Band::Green },
        Appearance::Dark,
        "#081310", "#121C18", "#20342C", "#0F1714",
        "#3FD9A6", "#86E8CB", "#070F0C",
    )
}

fn lagoon_light() -> ThemeVariant {
    half(
        &Persona { slug: "lagoon", name: "Lagoon", band: Band::Green },
        Appearance::Light,
        "#EAF6F5", "#DFEAE9", "#D1E2E0", "#F3FBFA",
        "#0C7C78", "#14837C", "#F0F8F7",
    )
}

fn lagoon_dark() -> ThemeVariant {
    half(
        &Persona { slug: "lagoon", name: "Lagoon", band: Band::Green },
        Appearance::Dark,
        "#06161A", "#101F22", "#1D3639", "#0C1A1D",
        "#35D6C4", "#82E6D8", "#061114",
    )
}

fn aqua_light() -> ThemeVariant {
    half(
        &Persona { slug: "aqua", name: "Aqua", band: Band::Green },
        Appearance::Light,
        "#E9F7FA", "#DEECEE", "#D0E1E6", "#F2FAFC",
        "#0D7D95", "#18788C", "#EFF8F9",
    )
}

fn aqua_dark() -> ThemeVariant {
    half(
        &Persona { slug: "aqua", name: "Aqua", band: Band::Green },
        Appearance::Dark,
        "#07151B", "#111E24", "#1E3640", "#0E1A20",
        "#2FDCE6", "#8BE9F0", "#071216",
    )
}

fn cobalt_light() -> ThemeVariant {
    half(
        &Persona { slug: "cobalt", name: "Cobalt", band: Band::Blue },
        Appearance::Light,
        "#EDF2FB", "#E2E8F2", "#D2DCEA", "#F6F9FD",
        "#2050CC", "#3452B8", "#F2F5FA",
    )
}

fn cobalt_dark() -> ThemeVariant {
    half(
        &Persona { slug: "cobalt", name: "Cobalt", band: Band::Blue },
        Appearance::Dark,
        "#0A1224", "#131C2E", "#25324A", "#101726",
        "#3D74FF", "#7FA8FF", "#08101C",
    )
}

fn midnight_light() -> ThemeVariant {
    half(
        &Persona { slug: "midnight", name: "Midnight", band: Band::Blue },
        Appearance::Light,
        "#ECF1FA", "#E1E7F0", "#D1DBE9", "#F5F8FC",
        "#2E5AAC", "#3D5490", "#F0F4FA",
    )
}

fn midnight_dark() -> ThemeVariant {
    half(
        &Persona { slug: "midnight", name: "Midnight", band: Band::Blue },
        Appearance::Dark,
        "#091127", "#121A31", "#232E4C", "#0E1529",
        "#5C9DFF", "#9BBEFF", "#070E20",
    )
}

fn indigo_light() -> ThemeVariant {
    half(
        &Persona { slug: "indigo", name: "Indigo", band: Band::Blue },
        Appearance::Light,
        "#F0F1FB", "#E6E7F1", "#D8DAE9", "#F9F9FD",
        "#4C58CC", "#5458B8", "#F3F4FA",
    )
}

fn indigo_dark() -> ThemeVariant {
    half(
        &Persona { slug: "indigo", name: "Indigo", band: Band::Blue },
        Appearance::Dark,
        "#10112B", "#191A36", "#2A2C48", "#141530",
        "#8C9BFF", "#B4BCFF", "#0B0C22",
    )
}

fn violet_light() -> ThemeVariant {
    half(
        &Persona { slug: "violet", name: "Violet", band: Band::Neutral },
        Appearance::Light,
        "#F6F2FE", "#ECE7F4", "#E4DDEF", "#FCFAFE",
        "#6F30C9", "#5C3AA8", "#FAF7FC",
    )
}

fn violet_dark() -> ThemeVariant {
    half(
        &Persona { slug: "violet", name: "Violet", band: Band::Neutral },
        Appearance::Dark,
        "#120E1F", "#1B172A", "#2C2440", "#161226",
        "#9B6BFF", "#C0A0FF", "#0D0A18",
    )
}

fn amethyst_light() -> ThemeVariant {
    half(
        &Persona { slug: "amethyst", name: "Amethyst", band: Band::Neutral },
        Appearance::Light,
        "#F8F3FD", "#EEE8F3", "#E6DCF0", "#FEFCFF",
        "#8A2BD0", "#6A34A0", "#FBF7FC",
    )
}

fn amethyst_dark() -> ThemeVariant {
    half(
        &Persona { slug: "amethyst", name: "Amethyst", band: Band::Neutral },
        Appearance::Dark,
        "#130D1C", "#1C1626", "#2E2440", "#171022",
        "#C26BF0", "#DBA8F8", "#0E0A16",
    )
}

fn magenta_light() -> ThemeVariant {
    half(
        &Persona { slug: "magenta", name: "Magenta", band: Band::Pink },
        Appearance::Light,
        "#FBEFF7", "#F1E4EC", "#E9D6E4", "#FEF8FB",
        "#C02694", "#942874", "#FAF2F7",
    )
}

fn magenta_dark() -> ThemeVariant {
    half(
        &Persona { slug: "magenta", name: "Magenta", band: Band::Pink },
        Appearance::Dark,
        "#160C16", "#1F151F", "#352433", "#1A111A",
        "#EF54C2", "#F79ADF", "#100810",
    )
}

fn sakura_light() -> ThemeVariant {
    half(
        &Persona { slug: "sakura", name: "Sakura", band: Band::Pink },
        Appearance::Light,
        "#FDF0F4", "#F3E5EB", "#EBDCE4", "#FFF9FB",
        "#C21D60", "#A03058", "#FBF4F7",
    )
}

fn sakura_dark() -> ThemeVariant {
    half(
        &Persona { slug: "sakura", name: "Sakura", band: Band::Pink },
        Appearance::Dark,
        "#170B10", "#20141A", "#382630", "#1B0F15",
        "#FF6FA5", "#FFA1C4", "#100709",
    )
}

fn silver_light() -> ThemeVariant {
    half(
        &Persona { slug: "silver", name: "Silver", band: Band::Neutral },
        Appearance::Light,
        "#F1F2F4", "#E8EAEC", "#DFE1E4", "#FAFAFB",
        "#4C5E6E", "#4A5666", "#F4F5F6",
    )
}

fn silver_dark() -> ThemeVariant {
    half(
        &Persona { slug: "silver", name: "Silver", band: Band::Neutral },
        Appearance::Dark,
        "#131518", "#1C1F22", "#2E3136", "#171A1D",
        "#9FB6CC", "#B8C4D2", "#0D0F11",
    )
}

fn cocoa_light() -> ThemeVariant {
    half(
        &Persona { slug: "cocoa", name: "Cocoa", band: Band::Neutral },
        Appearance::Light,
        "#FAF2E7", "#F1E8DC", "#E8DCCB", "#FEF9F2",
        "#8F571C", "#6E481A", "#FAF5EE",
    )
}

fn cocoa_dark() -> ThemeVariant {
    half(
        &Persona { slug: "cocoa", name: "Cocoa", band: Band::Neutral },
        Appearance::Dark,
        "#150E09", "#1E1712", "#362A1E", "#191210",
        "#DE9A50", "#EDBE85", "#0F0A07",
    )
}
