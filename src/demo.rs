use crate::diff::{DiffLine, FileDiff, LineKind};

/// Generate a 10-file demo diff with poems being edited/fixed by AI.
pub fn demo_files() -> Vec<FileDiff> {
    let diffs = vec![
        (
            "poems/frost.txt",
            vec![
                (LineKind::HunkHeader, "@@ -1,6 +1,6 @@"),
                (LineKind::Context, " Two roads diverged in a yellow wood,"),
                (LineKind::Remove, "-And sorry I could not travel both"),
                (LineKind::Add, "+And sorry I could not traverse both"),
                (LineKind::Context, " And be one traveler, long I stood"),
                (LineKind::Context, " And looked down one as far as I could"),
                (LineKind::Remove, "-To where it bent in the undergrowth;"),
                (LineKind::Add, "+To where it curved in the undergrowth;"),
            ],
        ),
        (
            "poems/dickinson.txt",
            vec![
                (LineKind::HunkHeader, "@@ -1,8 +1,9 @@"),
                (LineKind::Remove, "-Because I could not stop for death,"),
                (LineKind::Add, "+Because I could not stop for Death —"),
                (LineKind::Context, " He kindly stopped for me;"),
                (LineKind::Remove, "-The carriage held but just ourselves"),
                (LineKind::Add, "+The Carriage held but just Ourselves —"),
                (LineKind::Add, "+And Immortality."),
                (LineKind::Context, " "),
                (LineKind::Context, " We slowly drove, he knew no haste,"),
                (LineKind::Remove, "-And I had put away"),
                (LineKind::Add, "+And I had put away my labor"),
                (LineKind::Context, " My labor and my leisure too,"),
            ],
        ),
        (
            "poems/shakespeare_sonnet18.txt",
            vec![
                (LineKind::HunkHeader, "@@ -1,7 +1,7 @@"),
                (LineKind::Context, " Shall I compare thee to a summer's day?"),
                (LineKind::Remove, "-Thou art more lovely and more temprate."),
                (LineKind::Add, "+Thou art more lovely and more temperate."),
                (LineKind::Context, " Rough winds do shake the darling buds of May,"),
                (LineKind::Remove, "-And summer's lease hath all to short a date."),
                (LineKind::Add, "+And summer's lease hath all too short a date."),
                (LineKind::Context, " Sometime too hot the eye of heaven shines,"),
                (LineKind::Context, " And often is his gold complexion dimm'd;"),
            ],
        ),
        (
            "poems/poe_raven.txt",
            vec![
                (LineKind::HunkHeader, "@@ -1,10 +1,12 @@"),
                (LineKind::Context, " Once upon a midnight dreary,"),
                (LineKind::Remove, "-while I pondered weak and weary,"),
                (LineKind::Add, "+while I pondered, weak and weary,"),
                (LineKind::Context, " Over many a quaint and curious"),
                (LineKind::Remove, "-volume of forgotten lore,"),
                (LineKind::Add, "+volume of forgotten lore —"),
                (LineKind::Remove, "-While I nodded, nearly napping,"),
                (LineKind::Add, "+While I nodded, nearly napping, suddenly"),
                (LineKind::Add, "+there came a tapping,"),
                (LineKind::Context, " As of some one gently rapping,"),
                (LineKind::Remove, "-rapping at my chamber door"),
                (LineKind::Add, "+rapping at my chamber door."),
                (LineKind::Context, " \"'Tis some visitor,\" I muttered,"),
                (LineKind::Context, " \"tapping at my chamber door —"),
            ],
        ),
        (
            "poems/whitman.txt",
            vec![
                (LineKind::HunkHeader, "@@ -1,5 +1,6 @@"),
                (LineKind::Remove, "-O Captain! My Captain! our fearful trip is done,"),
                (LineKind::Add, "+O Captain! my Captain! our fearful trip is done,"),
                (LineKind::Context, " The ship has weather'd every rack,"),
                (LineKind::Remove, "-the prize we sought is won,"),
                (LineKind::Add, "+the prize we sought is won;"),
                (LineKind::Context, " The port is near, the bells I hear,"),
                (LineKind::Add, "+the people all exulting,"),
                (LineKind::Context, " While follow eyes the steady keel,"),
            ],
        ),
        (
            "poems/blake.txt",
            vec![
                (LineKind::HunkHeader, "@@ -1,8 +1,8 @@"),
                (LineKind::Context, " Tyger Tyger, burning bright,"),
                (LineKind::Remove, "-In the forests of the night;"),
                (LineKind::Add, "+In the forests of the night,"),
                (LineKind::Context, " What immortal hand or eye,"),
                (LineKind::Remove, "-Could frame thy fearful symmetry."),
                (LineKind::Add, "+Could frame thy fearful symmetry?"),
                (LineKind::Context, " "),
                (LineKind::Context, " In what distant deeps or skies,"),
                (LineKind::Remove, "-Burnt the fire of thine eyes"),
                (LineKind::Add, "+Burnt the fire of thine eyes?"),
            ],
        ),
        (
            "poems/keats_ode.txt",
            vec![
                (LineKind::HunkHeader, "@@ -1,6 +1,7 @@"),
                (LineKind::Context, " Thou still unravish'd bride of quietness,"),
                (LineKind::Remove, "-Thou foster-child of silence and slow time,"),
                (LineKind::Add, "+Thou foster-child of Silence and slow Time,"),
                (LineKind::Context, " Sylvan historian, who canst thus express"),
                (LineKind::Context, " A flowery tale more sweetly than our rhyme:"),
                (LineKind::Add, "+What leaf-fring'd legend haunts about thy shape"),
                (LineKind::Remove, "-What men or gods are these? What maidens loth?"),
                (LineKind::Add, "+Of deities or mortals, or of both,"),
            ],
        ),
        (
            "poems/wordsworth.txt",
            vec![
                (LineKind::HunkHeader, "@@ -1,7 +1,7 @@"),
                (LineKind::Remove, "-I wandered lonely as a Cloud"),
                (LineKind::Add, "+I wandered lonely as a cloud"),
                (LineKind::Context, " That floats on high o'er vales and hills,"),
                (LineKind::Context, " When all at once I saw a crowd,"),
                (LineKind::Remove, "-A host of golden Daffodils;"),
                (LineKind::Add, "+A host, of golden daffodils;"),
                (LineKind::Context, " Beside the lake, beneath the trees,"),
                (LineKind::Remove, "-Fluttering and dancing in the breez."),
                (LineKind::Add, "+Fluttering and dancing in the breeze."),
            ],
        ),
        (
            "poems/shelley.txt",
            vec![
                (LineKind::HunkHeader, "@@ -1,9 +1,10 @@"),
                (LineKind::Context, " I met a traveller from an antique land,"),
                (LineKind::Remove, "-Who said: \"Two vast and trunkless legs of stone"),
                (LineKind::Add, "+Who said — \"Two vast and trunkless legs of stone"),
                (LineKind::Context, " Stand in the desert. . . . Near them, on the sand,"),
                (LineKind::Remove, "-Half sunk a shattered visage lies, whose frown"),
                (LineKind::Add, "+Half sunk a shattered visage lies, whose frown,"),
                (LineKind::Context, " And wrinkled lip, and sneer of cold command,"),
                (LineKind::Context, " Tell that its sculptor well those passions read"),
                (LineKind::Remove, "-Which yet survive, stamped on these lifeless things,"),
                (LineKind::Add, "+Which yet survive, stamp'd on these lifeless things,"),
                (LineKind::Add, "+The hand that mock'd them, and the heart that fed;"),
            ],
        ),
        (
            "poems/cummings.txt",
            vec![
                (LineKind::HunkHeader, "@@ -1,6 +1,7 @@"),
                (LineKind::Remove, "-i carry your heart with me(i carry it in"),
                (LineKind::Add, "+i carry your heart with me (i carry it in"),
                (LineKind::Context, " my heart)i am never without it(anywhere"),
                (LineKind::Remove, "-i go you go, my dear;and whatever is done"),
                (LineKind::Add, "+i go you go,my dear; and whatever is done"),
                (LineKind::Context, " by only me is your doing,my darling)"),
                (LineKind::Add, "+                                   i fear"),
                (LineKind::Context, " no fate(for you are my fate,my sweet)"),
            ],
        ),
    ];

    diffs
        .into_iter()
        .map(|(filename, raw_lines)| {
            let mut old_line = 1u32;
            let mut new_line = 1u32;
            let mut additions = 0usize;
            let mut deletions = 0usize;

            let lines: Vec<DiffLine> = raw_lines
                .into_iter()
                .map(|(kind, content)| {
                    let (old_lineno, new_lineno) = match kind {
                        LineKind::Add => {
                            additions += 1;
                            let n = new_line;
                            new_line += 1;
                            (None, Some(n))
                        }
                        LineKind::Remove => {
                            deletions += 1;
                            let n = old_line;
                            old_line += 1;
                            (Some(n), None)
                        }
                        LineKind::Context => {
                            let (o, n) = (old_line, new_line);
                            old_line += 1;
                            new_line += 1;
                            (Some(o), Some(n))
                        }
                        LineKind::HunkHeader => (None, None),
                    };
                    DiffLine {
                        kind,
                        content: content.to_string(),
                        old_lineno,
                        new_lineno,
                    }
                })
                .collect();

            FileDiff {
                filename: filename.to_string(),
                additions,
                deletions,
                lines,
                styled_lines: Vec::new(),
            }
        })
        .collect()
}
