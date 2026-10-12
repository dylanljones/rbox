// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Database model update contracts expanded in the consuming test crate.
/// Verify master database tree updates, no-op USNs, and bulk rollback.
macro_rules! master_tree_updates {
    ($conn:ident, $model:ident, $new:ident, $leaf:expr) => {{
        let p = $new::folder("update parent").insert(&mut $conn).unwrap();
        let q = $new::folder("update destination")
            .insert(&mut $conn)
            .unwrap();
        let mut rows = Vec::new();
        for _ in 0..4 {
            rows.push(
                $new::folder("child")
                    .parent_id(&p.id)
                    .insert(&mut $conn)
                    .unwrap(),
            );
        }
        for source in 0..4 {
            for target in 0..4 {
                $crate::ordering::case(&mut $conn, |$conn| {
                    let mut row = rows[source].clone();
                    row.seq = target as i32 + 1;
                    row.update($conn)?;
                    let mut expected = rows.iter().map(|r| r.id.clone()).collect::<Vec<_>>();
                    let id = expected.remove(source);
                    expected.insert(target, id);
                    $crate::ordering::assert_order::<$model>($conn, &p.id, &expected);
                    Ok(())
                });
            }
        }
        $crate::ordering::case(&mut $conn, |$conn| {
            let mut row = rows[0].clone();
            row.parent_id = q.id.clone();
            row.seq = 1;
            row.update($conn)?;
            $crate::ordering::assert_order::<$model>(
                $conn,
                &p.id,
                &rows[1..].iter().map(|r| r.id.clone()).collect::<Vec<_>>(),
            );
            $crate::ordering::assert_order::<$model>($conn, &q.id, &[rows[0].id.clone()]);
            Ok(())
        });
        let usn = AgentRegistry::local_usn(&mut $conn).unwrap();
        let before = $model::ordered(&mut $conn, &p.id).unwrap();
        assert_eq!(rows[0].clone().update(&mut $conn).unwrap(), rows[0]);
        assert_eq!(AgentRegistry::local_usn(&mut $conn).unwrap(), usn);
        let mut cyclic = p.clone();
        cyclic.parent_id = rows[0].id.clone();
        assert!(cyclic.update(&mut $conn).is_err());
        let mut invalid_kind = p.clone();
        invalid_kind.attribute = $leaf;
        assert!(invalid_kind.update(&mut $conn).is_err());
        // The first insertion succeeds, then the invalid parent rejects the next one.
        assert!($new::insert_all(
            &mut $conn,
            vec![
                $new::folder("valid").parent_id(&p.id).seq(1),
                $new::folder("invalid").parent_id("missing-ordering-parent"),
            ]
        )
        .is_err());
        // Likewise, a bulk update must roll back an earlier successful move.
        let mut first = rows[0].clone();
        first.seq = 4;
        let mut invalid = rows[1].clone();
        invalid.seq = i32::MAX;
        assert!($model::update_all(&mut $conn, vec![first, invalid]).is_err());
        assert_eq!($model::ordered(&mut $conn, &p.id).unwrap(), before);
        assert_eq!(AgentRegistry::local_usn(&mut $conn).unwrap(), usn);
        let leaf = $new::new("leaf", $leaf).insert(&mut $conn).unwrap();
        assert!($new::folder("bad parent")
            .parent_id(&leaf.id)
            .insert(&mut $conn)
            .is_err());
        assert!($model::move_to(&mut $conn, &rows[0].id, Some(&leaf.id), None).is_err());
    }};
}

pub(crate) use master_tree_updates;
