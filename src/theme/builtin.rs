use super::{NamedTheme, ThemeColors, ThemeVariants};
use ratatui::style::Color;

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb(r, g, b)
}

pub fn get_builtin_themes() -> Vec<NamedTheme> {
    vec![
        NamedTheme {
            id: "default".into(),
            name: "Default".into(),
            is_builtin: true,
            variants: ThemeVariants {
                dark: Some(ThemeColors {
                    bg: rgb(22, 22, 30),
                    dialog_bg: rgb(35, 35, 45),
                    fg: rgb(230, 230, 240),
                    accent: rgb(138, 180, 248),
                    accent_secondary: rgb(187, 134, 252),
                    highlight: rgb(255, 203, 107),
                    muted: rgb(128, 128, 140),
                    success: rgb(129, 199, 132),
                    warning: rgb(255, 183, 77),
                    danger: rgb(239, 83, 80),
                    border: rgb(60, 60, 80),
                    selection_bg: rgb(50, 50, 70),
                    selection_fg: rgb(255, 255, 255),
                    graph_line: rgb(138, 180, 248),
                }),
                light: Some(ThemeColors {
                    bg: rgb(250, 250, 252),
                    dialog_bg: rgb(255, 255, 255),
                    fg: rgb(30, 30, 40),
                    accent: rgb(25, 118, 210),
                    accent_secondary: rgb(123, 31, 162),
                    highlight: rgb(255, 160, 0),
                    muted: rgb(140, 140, 150),
                    success: rgb(46, 125, 50),
                    warning: rgb(239, 108, 0),
                    danger: rgb(211, 47, 47),
                    border: rgb(200, 200, 210),
                    selection_bg: rgb(220, 230, 245),
                    selection_fg: rgb(0, 0, 0),
                    graph_line: rgb(25, 118, 210),
                }),
            },
        },
        NamedTheme {
            id: "catppuccin".into(),
            name: "Catppuccin".into(),
            is_builtin: true,
            variants: ThemeVariants {
                dark: Some(ThemeColors {
                    bg: rgb(30, 30, 46),             // #1e1e2e
                    dialog_bg: rgb(49, 50, 68),      // #313244
                    fg: rgb(205, 214, 244),          // #cdd6f4
                    accent: rgb(137, 180, 250),      // #89b4fa
                    accent_secondary: rgb(203, 166, 247), // #cba6f7
                    highlight: rgb(249, 226, 175),   // #f9e2af
                    muted: rgb(108, 112, 134),       // #6c7086
                    success: rgb(166, 227, 161),     // #a6e3a1
                    warning: rgb(250, 179, 135),     // #fab387
                    danger: rgb(243, 139, 168),      // #f38ba8
                    border: rgb(69, 71, 90),         // #45475a
                    selection_bg: rgb(88, 91, 112),  // #585b70
                    selection_fg: rgb(205, 214, 244), // #cdd6f4
                    graph_line: rgb(137, 180, 250),  // #89b4fa
                }),
                light: Some(ThemeColors {
                    bg: rgb(239, 241, 245),          // #eff1f5
                    dialog_bg: rgb(230, 233, 239),   // #e6e9ef
                    fg: rgb(76, 79, 105),            // #4c4f69
                    accent: rgb(30, 102, 245),       // #1e66f5
                    accent_secondary: rgb(136, 57, 239), // #8839ef
                    highlight: rgb(223, 142, 29),    // #df8e1d
                    muted: rgb(108, 111, 133),       // #6c6f85
                    success: rgb(64, 160, 43),       // #40a02b
                    warning: rgb(254, 100, 11),      // #fe640b
                    danger: rgb(210, 15, 57),        // #d20f39
                    border: rgb(188, 192, 204),      // #bcc0cc
                    selection_bg: rgb(172, 176, 190), // #acb0be
                    selection_fg: rgb(76, 79, 105),  // #4c4f69
                    graph_line: rgb(30, 102, 245),   // #1e66f5
                }),
            },
        },
        NamedTheme {
            id: "dracula".into(),
            name: "Dracula".into(),
            is_builtin: true,
            variants: ThemeVariants {
                dark: Some(ThemeColors {
                    bg: rgb(40, 42, 54),             // #282a36
                    dialog_bg: rgb(68, 71, 90),      // #44475a
                    fg: rgb(248, 248, 242),          // #f8f8f2
                    accent: rgb(189, 147, 249),      // #bd93f9
                    accent_secondary: rgb(255, 121, 198), // #ff79c6
                    highlight: rgb(241, 250, 140),   // #f1fa8c
                    muted: rgb(98, 114, 164),        // #6272a4
                    success: rgb(80, 250, 123),      // #50fa7b
                    warning: rgb(255, 184, 108),     // #ffb86c
                    danger: rgb(255, 85, 85),        // #ff5555
                    border: rgb(68, 71, 90),         // #44475a
                    selection_bg: rgb(68, 71, 90),   // #44475a
                    selection_fg: rgb(248, 248, 242), // #f8f8f2
                    graph_line: rgb(139, 233, 253),  // #8be9fd
                }),
                light: Some(ThemeColors {
                    bg: rgb(248, 248, 242),          // #f8f8f2
                    dialog_bg: rgb(255, 255, 255),   // #ffffff
                    fg: rgb(40, 42, 54),             // #282a36
                    accent: rgb(137, 89, 193),       // #8959c1
                    accent_secondary: rgb(200, 80, 150), // #c85096
                    highlight: rgb(180, 160, 60),    // #b4a03c
                    muted: rgb(98, 114, 164),        // #6272a4
                    success: rgb(40, 180, 80),       // #28b450
                    warning: rgb(200, 130, 60),      // #c8823c
                    danger: rgb(200, 50, 50),        // #c83232
                    border: rgb(200, 200, 210),      // #c8c8d2
                    selection_bg: rgb(220, 220, 230), // #dcdce6
                    selection_fg: rgb(40, 42, 54),   // #282a36
                    graph_line: rgb(80, 180, 200),   // #50b4c8
                }),
            },
        },
        NamedTheme {
            id: "nord".into(),
            name: "Nord".into(),
            is_builtin: true,
            variants: ThemeVariants {
                dark: Some(ThemeColors {
                    bg: rgb(46, 52, 64),             // #2e3440
                    dialog_bg: rgb(59, 66, 82),      // #3b4252
                    fg: rgb(216, 222, 233),          // #d8dee9
                    accent: rgb(136, 192, 208),      // #88c0d0
                    accent_secondary: rgb(180, 142, 173), // #b48ead
                    highlight: rgb(235, 203, 139),   // #ebcb8b
                    muted: rgb(76, 86, 106),         // #4c566a
                    success: rgb(163, 190, 140),     // #a3be8c
                    warning: rgb(208, 135, 112),     // #d08770
                    danger: rgb(191, 97, 106),       // #bf616a
                    border: rgb(67, 76, 94),         // #434c5e
                    selection_bg: rgb(76, 86, 106),  // #4c566a
                    selection_fg: rgb(236, 239, 244), // #eceff4
                    graph_line: rgb(129, 161, 193),  // #81a1c1
                }),
                light: Some(ThemeColors {
                    bg: rgb(236, 239, 244),          // #eceff4
                    dialog_bg: rgb(229, 233, 240),   // #e5e9f0
                    fg: rgb(59, 66, 82),             // #3b4252
                    accent: rgb(94, 129, 172),       // #5e81ac
                    accent_secondary: rgb(180, 142, 173), // #b48ead
                    highlight: rgb(235, 203, 139),   // #ebcb8b
                    muted: rgb(76, 86, 106),         // #4c566a
                    success: rgb(163, 190, 140),     // #a3be8c
                    warning: rgb(208, 135, 112),     // #d08770
                    danger: rgb(191, 97, 106),       // #bf616a
                    border: rgb(216, 222, 233),      // #d8dee9
                    selection_bg: rgb(216, 222, 233), // #d8dee9
                    selection_fg: rgb(46, 52, 64),   // #2e3440
                    graph_line: rgb(94, 129, 172),   // #5e81ac
                }),
            },
        },
        NamedTheme {
            id: "gruvbox".into(),
            name: "Gruvbox".into(),
            is_builtin: true,
            variants: ThemeVariants {
                dark: Some(ThemeColors {
                    bg: rgb(40, 40, 40),             // #282828
                    dialog_bg: rgb(60, 56, 54),      // #3c3836
                    fg: rgb(235, 219, 178),          // #ebdbb2
                    accent: rgb(131, 165, 152),      // #83a598
                    accent_secondary: rgb(211, 134, 155), // #d3869b
                    highlight: rgb(250, 189, 47),    // #fabd2f
                    muted: rgb(146, 131, 116),       // #928374
                    success: rgb(184, 187, 38),      // #b8bb26
                    warning: rgb(254, 128, 25),      // #fe8019
                    danger: rgb(251, 73, 52),        // #fb4934
                    border: rgb(80, 73, 69),         // #504945
                    selection_bg: rgb(102, 92, 84),  // #665c54
                    selection_fg: rgb(235, 219, 178), // #ebdbb2
                    graph_line: rgb(142, 192, 124),  // #8ec07c
                }),
                light: Some(ThemeColors {
                    bg: rgb(251, 241, 199),          // #fbf1c7
                    dialog_bg: rgb(242, 229, 188),   // #f2e5bc
                    fg: rgb(60, 56, 54),             // #3c3836
                    accent: rgb(69, 133, 136),       // #458588
                    accent_secondary: rgb(177, 98, 134), // #b16286
                    highlight: rgb(215, 153, 33),    // #d79921
                    muted: rgb(146, 131, 116),       // #928374
                    success: rgb(152, 151, 26),      // #98971a
                    warning: rgb(214, 93, 14),       // #d65d0e
                    danger: rgb(204, 36, 29),        // #cc241d
                    border: rgb(213, 196, 161),      // #d5c4a1
                    selection_bg: rgb(189, 174, 147), // #bdae93
                    selection_fg: rgb(60, 56, 54),   // #3c3836
                    graph_line: rgb(104, 157, 106),  // #689d6a
                }),
            },
        },
        NamedTheme {
            id: "tokyo-night".into(),
            name: "Tokyo Night".into(),
            is_builtin: true,
            variants: ThemeVariants {
                dark: Some(ThemeColors {
                    bg: rgb(26, 27, 38),             // #1a1b26
                    dialog_bg: rgb(36, 40, 59),      // #24283b
                    fg: rgb(192, 202, 245),          // #c0caf5
                    accent: rgb(122, 162, 247),      // #7aa2f7
                    accent_secondary: rgb(187, 154, 247), // #bb9af7
                    highlight: rgb(224, 175, 104),   // #e0af68
                    muted: rgb(86, 95, 137),         // #565f89
                    success: rgb(158, 206, 106),     // #9ece6a
                    warning: rgb(255, 158, 100),     // #ff9e64
                    danger: rgb(247, 118, 142),      // #f7768e
                    border: rgb(41, 46, 66),         // #292e42
                    selection_bg: rgb(59, 66, 97),   // #3b4261
                    selection_fg: rgb(192, 202, 245), // #c0caf5
                    graph_line: rgb(125, 207, 255),  // #7dcfff
                }),
                light: Some(ThemeColors {
                    bg: rgb(225, 226, 231),          // #e1e2e7
                    dialog_bg: rgb(255, 255, 255),   // #ffffff
                    fg: rgb(55, 96, 191),            // #3760bf
                    accent: rgb(46, 125, 233),       // #2e7de9
                    accent_secondary: rgb(152, 84, 241), // #9854f1
                    highlight: rgb(143, 105, 27),    // #8f691d
                    muted: rgb(132, 142, 179),       // #848eb3
                    success: rgb(56, 125, 68),       // #387d44
                    warning: rgb(150, 96, 48),       // #966030
                    danger: rgb(199, 74, 97),        // #c74a61
                    border: rgb(184, 187, 212),      // #b8bbd4
                    selection_bg: rgb(182, 191, 226), // #b6bfe2
                    selection_fg: rgb(55, 96, 191),  // #3760bf
                    graph_line: rgb(0, 123, 168),    // #007ba8
                }),
            },
        },
        NamedTheme {
            id: "solarized".into(),
            name: "Solarized".into(),
            is_builtin: true,
            variants: ThemeVariants {
                dark: Some(ThemeColors {
                    bg: rgb(0, 43, 54),              // #002b36
                    dialog_bg: rgb(7, 54, 66),       // #073642
                    fg: rgb(131, 148, 150),          // #839496
                    accent: rgb(38, 139, 210),       // #268bd2
                    accent_secondary: rgb(108, 113, 196), // #6c71c4
                    highlight: rgb(181, 137, 0),     // #b58900
                    muted: rgb(88, 110, 117),        // #586e75
                    success: rgb(133, 153, 0),       // #859900
                    warning: rgb(203, 75, 22),       // #cb4b16
                    danger: rgb(220, 50, 47),        // #dc322f
                    border: rgb(88, 110, 117),       // #586e75
                    selection_bg: rgb(7, 54, 66),    // #073642
                    selection_fg: rgb(147, 161, 161), // #93a1a1
                    graph_line: rgb(42, 161, 152),   // #2aa198
                }),
                light: Some(ThemeColors {
                    bg: rgb(253, 246, 227),          // #fdf6e3
                    dialog_bg: rgb(238, 232, 213),   // #eee8d5
                    fg: rgb(101, 123, 131),          // #657b83
                    accent: rgb(38, 139, 210),       // #268bd2
                    accent_secondary: rgb(108, 113, 196), // #6c71c4
                    highlight: rgb(181, 137, 0),     // #b58900
                    muted: rgb(147, 161, 161),       // #93a1a1
                    success: rgb(133, 153, 0),       // #859900
                    warning: rgb(203, 75, 22),       // #cb4b16
                    danger: rgb(220, 50, 47),        // #dc322f
                    border: rgb(147, 161, 161),      // #93a1a1
                    selection_bg: rgb(238, 232, 213), // #eee8d5
                    selection_fg: rgb(88, 110, 117), // #586e75
                    graph_line: rgb(42, 161, 152),   // #2aa198
                }),
            },
        },
        NamedTheme {
            id: "rose-pine".into(),
            name: "Rosé Pine".into(),
            is_builtin: true,
            variants: ThemeVariants {
                dark: Some(ThemeColors {
                    bg: rgb(25, 23, 36),             // #191724
                    dialog_bg: rgb(31, 29, 46),      // #1f1d2e
                    fg: rgb(224, 222, 244),          // #e0def4
                    accent: rgb(49, 116, 143),       // #31748f
                    accent_secondary: rgb(196, 167, 231), // #c4a7e7
                    highlight: rgb(246, 193, 119),   // #f6c177
                    muted: rgb(110, 106, 134),       // #6e6a86
                    success: rgb(156, 207, 216),     // #9ccfd8
                    warning: rgb(246, 193, 119),     // #f6c177
                    danger: rgb(235, 111, 146),      // #eb6f92
                    border: rgb(38, 35, 58),         // #26233a
                    selection_bg: rgb(64, 61, 82),   // #403d52
                    selection_fg: rgb(224, 222, 244), // #e0def4
                    graph_line: rgb(235, 188, 186),  // #ebbcba
                }),
                light: Some(ThemeColors {
                    bg: rgb(250, 244, 237),          // #faf4ed
                    dialog_bg: rgb(255, 250, 243),   // #fffaf3
                    fg: rgb(87, 82, 121),            // #575279
                    accent: rgb(40, 105, 131),       // #286983
                    accent_secondary: rgb(144, 122, 169), // #907aa9
                    highlight: rgb(234, 157, 52),    // #ea9d34
                    muted: rgb(152, 147, 165),       // #9893a5
                    success: rgb(86, 148, 159),      // #56949f
                    warning: rgb(234, 157, 52),      // #ea9d34
                    danger: rgb(180, 99, 122),       // #b4637a
                    border: rgb(242, 233, 222),      // #f2e9de
                    selection_bg: rgb(223, 218, 217), // #dfdad9
                    selection_fg: rgb(87, 82, 121),  // #575279
                    graph_line: rgb(215, 130, 126),  // #d7827e
                }),
            },
        },
        NamedTheme {
            id: "one-dark".into(),
            name: "One Dark".into(),
            is_builtin: true,
            variants: ThemeVariants {
                dark: Some(ThemeColors {
                    bg: rgb(40, 44, 52),             // #282c34
                    dialog_bg: rgb(53, 59, 69),      // #353b45
                    fg: rgb(171, 178, 191),          // #abb2bf
                    accent: rgb(97, 175, 239),       // #61afef
                    accent_secondary: rgb(198, 120, 221), // #c678dd
                    highlight: rgb(229, 192, 123),   // #e5c07b
                    muted: rgb(92, 99, 112),         // #5c6370
                    success: rgb(152, 195, 121),     // #98c379
                    warning: rgb(209, 154, 102),     // #d19a66
                    danger: rgb(224, 108, 117),      // #e06c75
                    border: rgb(99, 109, 131),       // #636d83
                    selection_bg: rgb(62, 68, 81),   // #3e4451
                    selection_fg: rgb(171, 178, 191), // #abb2bf
                    graph_line: rgb(86, 182, 194),   // #56b6c2
                }),
                light: Some(ThemeColors {
                    bg: rgb(250, 250, 250),          // #fafafa
                    dialog_bg: rgb(255, 255, 255),   // #ffffff
                    fg: rgb(56, 58, 66),             // #383a42
                    accent: rgb(64, 120, 242),       // #4078f2
                    accent_secondary: rgb(166, 38, 164), // #a626a4
                    highlight: rgb(193, 132, 1),     // #c18401
                    muted: rgb(160, 161, 167),       // #a0a1a7
                    success: rgb(80, 161, 79),       // #50a14f
                    warning: rgb(152, 104, 1),       // #986801
                    danger: rgb(228, 86, 73),        // #e45649
                    border: rgb(200, 200, 200),      // #c8c8c8
                    selection_bg: rgb(230, 230, 230), // #e6e6e6
                    selection_fg: rgb(56, 58, 66),   // #383a42
                    graph_line: rgb(1, 132, 188),    // #0184bc
                }),
            },
        },
        NamedTheme {
            id: "monokai".into(),
            name: "Monokai".into(),
            is_builtin: true,
            variants: ThemeVariants {
                dark: Some(ThemeColors {
                    bg: rgb(39, 40, 34),             // #272822
                    dialog_bg: rgb(53, 54, 47),      // #35362f
                    fg: rgb(248, 248, 242),          // #f8f8f2
                    accent: rgb(102, 217, 239),      // #66d9ef
                    accent_secondary: rgb(174, 129, 255), // #ae81ff
                    highlight: rgb(230, 219, 116),   // #e6db74
                    muted: rgb(117, 113, 94),        // #75715e
                    success: rgb(166, 226, 46),      // #a6e22e
                    warning: rgb(253, 151, 31),      // #fd971f
                    danger: rgb(249, 38, 114),       // #f92672
                    border: rgb(73, 72, 62),         // #49483e
                    selection_bg: rgb(73, 72, 62),   // #49483e
                    selection_fg: rgb(248, 248, 242), // #f8f8f2
                    graph_line: rgb(102, 217, 239),  // #66d9ef
                }),
                light: Some(ThemeColors {
                    bg: rgb(253, 253, 253),          // #fdfdfd
                    dialog_bg: rgb(255, 255, 255),   // #ffffff
                    fg: rgb(39, 40, 34),             // #272822
                    accent: rgb(41, 171, 184),       // #29abb8
                    accent_secondary: rgb(126, 87, 194), // #7e57c2
                    highlight: rgb(156, 142, 36),    // #9c8e24
                    muted: rgb(117, 113, 94),        // #75715e
                    success: rgb(104, 159, 56),      // #689f38
                    warning: rgb(200, 117, 15),      // #c8750f
                    danger: rgb(194, 24, 91),        // #c2185b
                    border: rgb(200, 200, 190),      // #c8c8be
                    selection_bg: rgb(230, 230, 220), // #e6e6dc
                    selection_fg: rgb(39, 40, 34),   // #272822
                    graph_line: rgb(41, 171, 184),   // #29abb8
                }),
            },
        },
    ]
}
