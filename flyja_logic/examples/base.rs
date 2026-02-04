use flyja_logic::{Id, InsertWay, Position, Size, SizeAndPos, TopElementMap};

const DISPLAY_SIZE: SizeAndPos = SizeAndPos {
    size: Size {
        width: 1980.,
        height: 1080.,
    },
    position: Position { x: 0., y: 0. },
};

fn main() {
    let mut element_map = TopElementMap::new(DISPLAY_SIZE);
    // ID: 0
    // -------------
    // |           |
    // |           |
    // |     0     |
    // |           |
    // |           |
    // -------------
    let id = Id::unique();
    element_map
        .insert_new(
            id,
            Id::MAIN,
            InsertWay::Vertical,
            &mut |in_id, size_and_pos| {
                assert_eq!(in_id, id);
                assert_eq!(size_and_pos, DISPLAY_SIZE);
            },
        )
        .unwrap();
    dbg!(&element_map);
    // ID: 1
    // ------------
    // |          |
    // |    0     |
    // |          |
    // ------------
    // |          |
    // |    1     |
    // |          |
    // ------------
    element_map
        .insert_new(
            Id::unique(),
            Id(0),
            InsertWay::Vertical,
            &mut |id, size_and_pos| match id {
                Id(0) => assert_eq!(
                    size_and_pos,
                    SizeAndPos {
                        position: Position { x: 0., y: 0. },
                        size: Size {
                            width: 1980.,
                            height: 540.
                        }
                    }
                ),
                Id(1) => assert_eq!(
                    size_and_pos,
                    SizeAndPos {
                        position: Position { x: 0., y: 540. },
                        size: Size {
                            width: 1980.,
                            height: 540.
                        }
                    }
                ),
                _ => unreachable!(),
            },
        )
        .unwrap();
    dbg!(&element_map);
    // ID: 2
    // ------------
    // |          |
    // |    0     |
    // ------------
    // |          |
    // |    2     |
    // ------------
    // |          |
    // |    1     |
    // ------------
    element_map
        .insert_new(
            Id::unique(),
            Id(0),
            InsertWay::Vertical,
            &mut |id, size| match id {
                Id(0) => assert_eq!(
                    size,
                    SizeAndPos {
                        position: Position { x: 0., y: 0. },
                        size: Size {
                            width: 1980.,
                            height: 270.
                        }
                    }
                ),
                Id(1) => assert_eq!(
                    size,
                    SizeAndPos {
                        position: Position { x: 0., y: 720. },
                        size: Size {
                            width: 1980.,
                            height: 270.
                        }
                    }
                ),
                Id(2) => assert_eq!(
                    size,
                    SizeAndPos {
                        position: Position { x: 0., y: 360. },
                        size: Size {
                            width: 1980.,
                            height: 540.
                        }
                    }
                ),
                _ => unreachable!(),
            },
        )
        .unwrap();
    dbg!(&element_map);
    // ID: 3
    // ------------
    // | 0  |  3  |
    // |    |     |
    // ------------
    // |          |
    // |    2     |
    // ------------
    // |          |
    // |    1     |
    // ------------
    element_map
        .insert_new(
            Id::unique(),
            Id(0),
            InsertWay::Horizontal,
            &mut |id, size_and_pos| match id {
                Id(0) => assert_eq!(
                    size_and_pos,
                    SizeAndPos {
                        position: Position { x: 0., y: 0. },
                        size: Size {
                            width: 990.,
                            height: 270.
                        }
                    }
                ),
                Id(1) => assert_eq!(
                    size_and_pos,
                    SizeAndPos {
                        position: Position { x: 0., y: 720. },
                        size: Size {
                            width: 1980.,
                            height: 540.
                        }
                    }
                ),
                Id(2) => assert_eq!(
                    size_and_pos,
                    SizeAndPos {
                        position: Position { x: 0., y: 360. },
                        size: Size {
                            width: 1980.,
                            height: 270.
                        }
                    }
                ),
                Id(3) => assert_eq!(
                    size_and_pos,
                    SizeAndPos {
                        position: Position { x: 990., y: 0. },
                        size: Size {
                            width: 990.,
                            height: 270.
                        }
                    }
                ),
                _ => unreachable!(),
            },
        )
        .unwrap();
    dbg!(&element_map);

    // ======= delete ===========
    //  ------------
    // |    |     |
    // |    |     |
    // | 0  |  3  |
    // |    |     |
    // ------------
    // |          |
    // |          |
    // |          |
    // |    1     |
    // ------------
    element_map
        .delete(Id(2), &mut |id, size_and_pos| match id {
            Id(0) => assert_eq!(
                size_and_pos,
                SizeAndPos {
                    position: Position { x: 0., y: 0. },
                    size: Size {
                        width: 990.,
                        height: 540.
                    }
                }
            ),
            Id(1) => assert_eq!(
                size_and_pos,
                SizeAndPos {
                    position: Position { x: 0., y: 540. },
                    size: Size {
                        width: 1980.,
                        height: 540.
                    }
                }
            ),
            Id(3) => assert_eq!(
                size_and_pos,
                SizeAndPos {
                    position: Position { x: 990., y: 0. },
                    size: Size {
                        width: 990.,
                        height: 540.
                    }
                }
            ),
            _ => unreachable!(),
        })
        .unwrap();
    dbg!(&element_map);
    //  ------------
    // |          |
    // |          |
    // |    3     |
    // |          |
    // ------------
    // |          |
    // |          |
    // |          |
    // |    1     |
    // ------------
    element_map
        .delete(Id(0), &mut |id, size_and_pos| match id {
            Id(1) => assert_eq!(
                size_and_pos,
                SizeAndPos {
                    position: Position { x: 0., y: 540. },
                    size: Size {
                        width: 1980.,
                        height: 540.
                    }
                }
            ),
            Id(3) => assert_eq!(
                size_and_pos,
                SizeAndPos {
                    position: Position { x: 0., y: 0. },
                    size: Size {
                        width: 1980.,
                        height: 540.
                    }
                }
            ),
            _ => unreachable!(),
        })
        .unwrap();
    dbg!(&element_map);
    //  ------------
    // |          |
    // |          |
    // |    3     |
    // |          |
    // ------------
    element_map
        .delete(Id(1), &mut |id, size_and_pos| match id {
            Id(3) => assert_eq!(
                size_and_pos,
                SizeAndPos {
                    position: Position { x: 0., y: 0. },
                    size: Size {
                        width: 1980.,
                        height: 1080.
                    }
                }
            ),
            _ => unreachable!(),
        })
        .unwrap();
    dbg!(&element_map);
    //  ------------
    // |          |
    // |          |
    // |  EMPTY   |
    // |          |
    // ------------
    element_map
        .delete(Id(3), &mut |_, _| unreachable!())
        .unwrap();
    dbg!(&element_map);
}
