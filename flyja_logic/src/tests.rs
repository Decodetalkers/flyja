use super::*;
const DISPLAY_SIZE: SizeAndPos = SizeAndPos {
    size: Size {
        width: 1980.,
        height: 1080.,
    },
    position: Position { x: 0., y: 0. },
};

#[test]
fn drag_drop() {
    let mut element_map = TopElementMap::new(DISPLAY_SIZE);
    // ID: 0
    // -------------
    // |           |
    // |           |
    // |     0     |
    // |           |
    // |           |
    // -------------
    let id = Id::MAIN;
    element_map
        .insert(id, Id::MAIN, Direction::Top, &mut |in_id, size_and_pos| {
            assert_eq!(in_id, id);
            assert_eq!(size_and_pos, DISPLAY_SIZE);
        })
        .expect("Insert should succeeded");
    // ID: 1
    // ------------
    // |          |
    // |    1     |
    // |          |
    // ------------
    // |          |
    // |    0     |
    // |          |
    // ------------
    let id = id.next();
    element_map
        .insert(
            id,
            Id(0),
            Direction::Top,
            &mut |id, size_and_pos| match id {
                Id(1) => assert_eq!(
                    size_and_pos,
                    SizeAndPos {
                        position: Position { x: 0., y: 0. },
                        size: Size {
                            width: 1980.,
                            height: 540.
                        }
                    }
                ),
                Id(0) => assert_eq!(
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
        .expect("Insert should succeeded");
    // ID: 1
    // -----------------------
    // |         |           |
    // |   1     |     0     |
    // |         |           |
    // -----------------------
    let mut times = 0;
    element_map
        .drag_and_drop(Id(1), Id(0), Direction::Left, &mut |id, size_and_pos| {
            match id {
                Id(0) => assert_eq!(
                    size_and_pos,
                    SizeAndPos {
                        size: Size {
                            width: 990.,
                            height: 1080.
                        },
                        position: Position { x: 990., y: 0. }
                    }
                ),
                Id(1) => assert_eq!(
                    size_and_pos,
                    SizeAndPos {
                        size: Size {
                            width: 990.,
                            height: 1080.
                        },
                        position: Position { x: 0., y: 0. }
                    }
                ),
                _ => unreachable!(),
            }
            times += 1;
        })
        .expect("Should succeeded");
    assert_eq!(times, 2);
}

#[test]
fn drag_and_resize_test() {
    let mut element_map = TopElementMap::new(DISPLAY_SIZE);
    // ID: 0
    // -------------
    // |           |
    // |           |
    // |     0     |
    // |           |
    // |           |
    // -------------
    let id = Id::MAIN;
    element_map
        .insert(id, Id::MAIN, Direction::Top, &mut |in_id, size_and_pos| {
            assert_eq!(in_id, id);
            assert_eq!(size_and_pos, DISPLAY_SIZE);
        })
        .expect("Insert should succeeded");
    // ID: 1
    // ------------
    // |          |
    // |    1     |
    // |          |
    // ------------
    // |          |
    // |    0     |
    // |          |
    // ------------
    let id = id.next();
    element_map
        .insert(
            id,
            Id(0),
            Direction::Top,
            &mut |id, size_and_pos| match id {
                Id(1) => assert_eq!(
                    size_and_pos,
                    SizeAndPos {
                        position: Position { x: 0., y: 0. },
                        size: Size {
                            width: 1980.,
                            height: 540.
                        }
                    }
                ),
                Id(0) => assert_eq!(
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
        .expect("Insert should succeeded");
    let mut times = 0;
    element_map
        .drag_resize(-270., Direction::Bottom, Id(1), &mut |id, size_and_pos| {
            match id {
                Id(0) => assert_eq!(
                    size_and_pos,
                    SizeAndPos {
                        size: Size {
                            width: 1980.,
                            height: 810.
                        },
                        position: Position { x: 0., y: 270. }
                    }
                ),
                Id(1) => assert_eq!(
                    size_and_pos,
                    SizeAndPos {
                        size: Size {
                            width: 1980.,
                            height: 270.
                        },
                        position: Position { x: 0., y: 0. }
                    }
                ),
                _ => unreachable!(),
            }
            times += 1;
        })
        .expect("Should succeeded");
    assert_eq!(times, 2);
}

#[test]
fn insert_test() {
    let mut element_map = TopElementMap::new(DISPLAY_SIZE);
    // ID: 0
    // -------------
    // |           |
    // |           |
    // |     0     |
    // |           |
    // |           |
    // -------------
    let id = Id::MAIN;
    element_map
        .insert(id, Id::MAIN, Direction::Top, &mut |in_id, size_and_pos| {
            assert_eq!(in_id, id);
            assert_eq!(size_and_pos, DISPLAY_SIZE);
        })
        .expect("Insert should succeeded");
    // ID: 1
    // ------------
    // |          |
    // |    1     |
    // |          |
    // ------------
    // |          |
    // |    0     |
    // |          |
    // ------------
    let id = id.next();
    element_map
        .insert(
            id,
            Id(0),
            Direction::Top,
            &mut |id, size_and_pos| match id {
                Id(1) => assert_eq!(
                    size_and_pos,
                    SizeAndPos {
                        position: Position { x: 0., y: 0. },
                        size: Size {
                            width: 1980.,
                            height: 540.
                        }
                    }
                ),
                Id(0) => assert_eq!(
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
        .expect("Insert should succeeded");
    // ID: 2
    // ------------
    // |          |
    // |          |
    // |          |
    // |    1     |
    // ------------
    // |          |
    // |    2     |
    // ------------
    // |          |
    // |    0     |
    // ------------
    let id = id.next();
    element_map
        .insert(id, Id(0), Direction::Top, &mut |id, size| match id {
            Id(1) => assert_eq!(
                size,
                SizeAndPos {
                    position: Position { x: 0., y: 0. },
                    size: Size {
                        width: 1980.,
                        height: 540.
                    }
                }
            ),
            Id(0) => assert_eq!(
                size,
                SizeAndPos {
                    position: Position { x: 0., y: 810. },
                    size: Size {
                        width: 1980.,
                        height: 270.
                    }
                }
            ),
            Id(2) => assert_eq!(
                size,
                SizeAndPos {
                    position: Position { x: 0., y: 540. },
                    size: Size {
                        width: 1980.,
                        height: 270.
                    }
                }
            ),
            _ => unreachable!(),
        })
        .expect("Insert should succeeded");
    dbg!(&element_map);
    // ID: 3
    // ------------
    // |    1     |
    // |          |
    // |          |
    // |          |
    // |          |
    // ------------
    // |          |
    // |    2     |
    // ------------
    // |    |     |
    // | 3  |  0  |
    // ------------
    let id = id.next();
    element_map
        .insert(
            id,
            Id(0),
            Direction::Left,
            &mut |id, size_and_pos| match id {
                Id(0) => assert_eq!(
                    size_and_pos,
                    SizeAndPos {
                        position: Position { x: 990., y: 810. },
                        size: Size {
                            width: 990.,
                            height: 270.
                        }
                    }
                ),
                Id(1) => assert_eq!(
                    size_and_pos,
                    SizeAndPos {
                        position: Position { x: 0., y: 0. },
                        size: Size {
                            width: 1980.,
                            height: 540.
                        }
                    }
                ),
                Id(2) => assert_eq!(
                    size_and_pos,
                    SizeAndPos {
                        position: Position { x: 0., y: 540. },
                        size: Size {
                            width: 1980.,
                            height: 270.
                        }
                    }
                ),
                Id(3) => assert_eq!(
                    size_and_pos,
                    SizeAndPos {
                        position: Position { x: 0., y: 810. },
                        size: Size {
                            width: 990.,
                            height: 270.
                        }
                    }
                ),
                _ => unreachable!(),
            },
        )
        .expect("Insert should succeeded");
}

#[test]
fn insert_new_and_delete() {
    let mut element_map = TopElementMap::new(DISPLAY_SIZE);
    // ID: 0
    // -------------
    // |           |
    // |           |
    // |     0     |
    // |           |
    // |           |
    // -------------
    let id = Id::MAIN;
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
        .expect("Insert should succeeded");
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
    let id = id.next();
    element_map
        .insert_new(
            id,
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
        .expect("Insert should succeeded");
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
    let id = id.next();
    element_map
        .insert_new(id, Id(0), InsertWay::Vertical, &mut |id, size| match id {
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
        })
        .expect("Insert should succeeded");
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
    let id = id.next();
    element_map
        .insert_new(
            id,
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
        .expect("Insert should succeeded");

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
        .expect("Should work");
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
        .expect("Should work");
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
        .expect("Should work");
    //  ------------
    // |          |
    // |          |
    // |  EMPTY   |
    // |          |
    // ------------
    element_map
        .delete(Id(3), &mut |_, _| unreachable!())
        .expect("Should work");
}

#[test]
fn remap_test() {
    let remap_size_pos = SizeAndPos {
        size: Size {
            width: 2880.,
            height: 1800.,
        },
        position: Position { x: 0., y: 0. },
    };
    // === simple remap ===
    let mut element_map = TopElementMap::new(DISPLAY_SIZE);
    let id = Id::MAIN;
    let _ = element_map.insert_new(id, Id::MAIN, InsertWay::Vertical, &mut |_, _| {});

    element_map.remap(remap_size_pos, &mut |id, size_pos| {
        assert_eq!(id, Id(0));
        assert_eq!(size_pos, remap_size_pos);
    });

    // === Vertical remap ===
    let mut element_map = TopElementMap::new(DISPLAY_SIZE);
    let id = Id::MAIN;
    let _ = element_map.insert_new(id, Id::MAIN, InsertWay::Vertical, &mut |_, _| {});
    let id_old = id;
    let id = id.next();
    let _ = element_map.insert_new(id, id_old, InsertWay::Vertical, &mut |_, _| {});

    element_map.remap(remap_size_pos, &mut |id, size_pos| match id {
        Id(0) => assert_eq!(
            size_pos,
            SizeAndPos {
                size: Size {
                    width: 2880.,
                    height: 900.
                },
                position: Position { x: 0., y: 0. }
            }
        ),
        Id(1) => assert_eq!(
            size_pos,
            SizeAndPos {
                size: Size {
                    width: 2880.,
                    height: 900.
                },
                position: Position { x: 0., y: 900. }
            }
        ),
        _ => unreachable!(),
    });

    // === Horizontal remap ===
    let mut element_map = TopElementMap::new(DISPLAY_SIZE);
    let id = Id::MAIN;
    let _ = element_map.insert_new(id, Id::MAIN, InsertWay::Vertical, &mut |_, _| {});
    let id_old = id;
    let id = id.next();
    let _ = element_map.insert_new(id, id_old, InsertWay::Horizontal, &mut |_, _| {});

    element_map.remap(remap_size_pos, &mut |id, size_pos| match id {
        Id(0) => assert_eq!(
            size_pos,
            SizeAndPos {
                size: Size {
                    width: 1440.,
                    height: 1800.
                },
                position: Position { x: 0., y: 0. }
            }
        ),
        Id(1) => assert_eq!(
            size_pos,
            SizeAndPos {
                size: Size {
                    width: 1440.,
                    height: 1800.
                },
                position: Position { x: 1440., y: 0. }
            }
        ),
        _ => unreachable!(),
    });
}

#[test]
fn swap_test() {
    // == Vertical test ==
    let mut element_map = TopElementMap::new(DISPLAY_SIZE);
    let id = Id::MAIN;
    let _ = element_map.insert_new(id, Id::MAIN, InsertWay::Vertical, &mut |_, _| {});
    let id_old = id;
    let id = id.next();
    let _ = element_map.insert_new(id, id_old, InsertWay::Vertical, &mut |_, _| {});
    element_map
        .swap(Id(0), Id(1), &mut |id, size_pos| match id {
            Id(0) => assert_eq!(
                size_pos,
                SizeAndPos {
                    size: Size {
                        width: 1980.,
                        height: 540.
                    },
                    position: Position { x: 0., y: 540. },
                }
            ),
            Id(1) => assert_eq!(
                size_pos,
                SizeAndPos {
                    size: Size {
                        width: 1980.,
                        height: 540.
                    },
                    position: Position { x: 0., y: 0. },
                }
            ),
            _ => unreachable!(),
        })
        .expect("Should ok");
    // == Horizontal test ==
    let mut element_map = TopElementMap::new(DISPLAY_SIZE);
    let id = Id::MAIN;
    let _ = element_map.insert_new(id, Id::MAIN, InsertWay::Horizontal, &mut |_, _| {});
    let id_old = id;
    let id = id.next();
    let _ = element_map.insert_new(id, id_old, InsertWay::Horizontal, &mut |_, _| {});
    element_map
        .swap(Id(0), Id(1), &mut |id, size_pos| match id {
            Id(0) => assert_eq!(
                size_pos,
                SizeAndPos {
                    size: Size {
                        width: 990.,
                        height: 1080.
                    },
                    position: Position { x: 990., y: 0. },
                }
            ),
            Id(1) => assert_eq!(
                size_pos,
                SizeAndPos {
                    size: Size {
                        width: 990.,
                        height: 1080.
                    },
                    position: Position { x: 0., y: 0. },
                }
            ),
            _ => unreachable!(),
        })
        .expect("Should ok");
}
