// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! OneLibrary model update contracts.

macro_rules! one_tree_updates {
    ($conn:ident, $model:ident, $new:ident, $leaf:expr) => {{
        let parent = $new::folder("parent").insert(&mut $conn).unwrap();
        let first = $new::folder("first")
            .parent_id(parent.id)
            .insert(&mut $conn)
            .unwrap();
        assert!($new::insert_all(
            &mut $conn,
            vec![
                $new::folder("valid")
                    .parent_id(parent.id)
                    .seq($model::START_SEQ),
                $new::folder("invalid").parent_id(i64::MAX),
            ]
        )
        .is_err());
        $crate::ordering::assert_order::<$model>(&mut $conn, &parent.id, &[first.id]);
        let leaf = $new::new("leaf", $leaf).insert(&mut $conn).unwrap();
        assert!($new::folder("bad parent")
            .parent_id(leaf.id)
            .insert(&mut $conn)
            .is_err());
        assert!($model::move_to(&mut $conn, &first.id, Some(&leaf.id), None).is_err());
    }};
}

pub(crate) use one_tree_updates;
