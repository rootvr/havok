mod climber;
mod constant;
mod dice;
pub mod error;
mod evaluator;
mod parser;
pub mod roll;
pub mod solver;

#[cfg(test)]
mod tests {
    use crate::roll::Kind;
    use crate::roll::Source;
    use crate::solver::Solver;

    pub struct MockIter<'a, T: Iterator<Item = u64>> {
        pub iter: &'a mut T,
    }

    impl<T: Iterator<Item = u64>> Source for MockIter<'_, T> {
        fn throw(&mut self, sides: u64) -> u64 {
            match self.iter.next() {
                Some(value) => {
                    if value > sides {
                        panic!("Tried to return {} for a {} sided dice", value, sides)
                    }
                    println!("Dice {}", value);
                    value
                }
                None => panic!("Iterator out of values"),
            }
        }
    }

    #[test]
    fn get_repeat_test() {
        let r = Solver::new("(2d6 + 6) ^ 8 : test").unwrap();
        let roll_mock = vec![3, 5, 3, 5, 3, 5, 3, 5, 3, 5, 3, 5, 3, 5, 3, 5];
        let roll_res = r
            .solve_with_source(&mut MockIter {
                iter: &mut roll_mock.into_iter(),
            })
            .unwrap();
        match roll_res.get_result() {
            Kind::Single(_) => unreachable!(),
            Kind::Multi(rep) => {
                assert_eq!(8, rep.len());
                for res in rep.iter() {
                    assert_eq!(14, res.get_total());
                }
            }
        }
        eprintln!();
        for res in roll_res.as_multi().unwrap().iter() {
            eprintln!("{}", res)
        }

        eprintln!();
        eprintln!("{}", roll_res);
    }

    #[test]
    fn get_repeat_sort_test() {
        let r = Solver::new("(2d6 + 6) ^# 8 : test").unwrap();
        let roll_mock = vec![3, 5, 1, 1, 6, 5, 3, 5, 4, 5, 2, 4, 3, 5, 1, 2];
        let mut expected = roll_mock
            .as_slice()
            .chunks(2)
            .map(|two| two[0] as i64 + two[1] as i64 + 6)
            .collect::<Vec<_>>();
        expected.sort_unstable();
        let roll_res = r
            .solve_with_source(&mut MockIter {
                iter: &mut roll_mock.into_iter(),
            })
            .unwrap();
        match roll_res.get_result() {
            Kind::Single(_) => unreachable!(),
            Kind::Multi(rep) => {
                assert_eq!(8, rep.len());

                let res_vec = rep.iter().map(|r| r.get_total()).collect::<Vec<_>>();
                assert_eq!(expected, res_vec);
            }
        };
        eprintln!("{}", roll_res);
    }

    #[test]
    fn get_repeat_sum_test() {
        let r = Solver::new("(2d6 + 6) ^+ 2 : test").unwrap();
        let roll_mock = vec![3, 5, 4, 2];
        let expected = roll_mock
            .as_slice()
            .chunks(2)
            .map(|two| two[0] as i64 + two[1] as i64 + 6)
            .collect::<Vec<_>>();
        let expected: i64 = expected.iter().sum();
        let roll_res = r
            .solve_with_source(&mut MockIter {
                iter: &mut roll_mock.into_iter(),
            })
            .unwrap();
        match roll_res.get_result() {
            Kind::Single(_) => unreachable!(),
            Kind::Multi(rep) => {
                assert_eq!(2, rep.len());
                assert_eq!(expected, rep.get_total().unwrap());
            }
        }
        eprintln!();
        eprintln!("{}", roll_res);
    }

    #[test]
    fn get_single_test() {
        let r = Solver::new("2d6 + 6 : test").unwrap();
        let roll_mock = vec![3, 5];
        let expected = roll_mock
            .as_slice()
            .chunks(2)
            .map(|two| two[0] as i64 + two[1] as i64)
            .collect::<Vec<_>>();
        let expected = expected.iter().sum::<i64>() + 6;
        let roll_res = r
            .solve_with_source(&mut MockIter {
                iter: &mut roll_mock.into_iter(),
            })
            .unwrap();
        match roll_res.get_result() {
            Kind::Single(res) => assert_eq!(expected, res.get_total()),
            Kind::Multi(_) => unreachable!(),
        }
        eprintln!();
        eprintln!("{}", roll_res.as_single().unwrap());
    }

    #[test]
    fn one_value_test() {
        let r = Solver::new("20").unwrap();
        let res = r.solve().unwrap();
        let res = res.get_result();
        if let Kind::Single(res) = res {
            assert_eq!(20, res.get_total());
        } else {
            assert!(false);
        }
    }

    #[test]
    fn one_dice_test() {
        let r = Solver::new("d20").unwrap();
        let roll_mock = vec![8];
        let res = r
            .solve_with_source(&mut MockIter {
                iter: &mut roll_mock.into_iter(),
            })
            .unwrap();
        let res = res.get_result();
        if let Kind::Single(res) = res {
            assert_eq!(8, res.get_total());
        } else {
            unreachable!();
        }
    }

    #[test]
    fn float_mul_test() {
        let r = Solver::new("20 * 1.5").unwrap();
        let res = r.solve().unwrap();
        let res = res.get_result();
        if let Kind::Single(res) = res {
            assert_eq!(30, res.get_total());
        } else {
            unreachable!()
        }
    }

    #[test]
    fn float_signed_mul_test() {
        let r = Solver::new("20 * +1.5").unwrap();
        let res = r.solve().unwrap();
        let res = res.get_result();
        if let Kind::Single(res) = res {
            assert_eq!(30, res.get_total());
        } else {
            unreachable!()
        }
    }

    #[test]
    fn float_neg_signed_mul_test() {
        let r = Solver::new("20 * -1.5").unwrap();
        let res = r.solve().unwrap();
        let res = res.get_result();
        if let Kind::Single(res) = res {
            assert_eq!(-30, res.get_total());
        } else {
            unreachable!()
        }
    }

    #[test]
    fn float_add_test() {
        let r = Solver::new("20 + 1.5").unwrap();
        let res = r.solve().unwrap();
        let res = res.get_result();
        if let Kind::Single(res) = res {
            assert_eq!(21, res.get_total());
        } else {
            unreachable!()
        }
    }

    #[test]
    fn float_signed_add_test() {
        let r = Solver::new("20 + +1.5").unwrap();
        let res = r.solve().unwrap();
        let res = res.get_result();
        if let Kind::Single(res) = res {
            assert_eq!(21, res.get_total());
        } else {
            unreachable!()
        }
    }

    #[test]
    fn float_neg_signed_add_test() {
        let r = Solver::new("20 + -1.5").unwrap();
        let res = r.solve().unwrap();
        let res = res.get_result();
        if let Kind::Single(res) = res {
            assert_eq!(18, res.get_total());
        } else {
            unreachable!()
        }
    }

    #[test]
    fn signed_add_test() {
        let r = Solver::new("20 + +5").unwrap();
        let res = r.solve().unwrap();
        let res = res.get_result();
        if let Kind::Single(res) = res {
            assert_eq!(25, res.get_total());
        } else {
            unreachable!()
        }
    }

    #[test]
    fn signed_neg_add_test() {
        let r = Solver::new("20 + -5").unwrap();
        let res = r.solve().unwrap();
        let res = res.get_result();
        if let Kind::Single(res) = res {
            assert_eq!(15, res.get_total());
        } else {
            unreachable!()
        }
    }

    #[test]
    fn counting_roller_test() {
        let r = Solver::new("3d6").unwrap();
        let rolls = vec![3, 6, 3];
        let res = r
            .solve_with_source(&mut MockIter {
                iter: &mut rolls.into_iter(),
            })
            .unwrap();
        let res = res.get_result();
        if let Kind::Single(res) = res {
            assert_eq!(res.get_total(), 12);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn target_number_test() {
        let r = Solver::new("10d10 t7").unwrap();
        let res = r
            .solve_with_source(&mut MockIter { iter: &mut (1..11) })
            .unwrap();
        println!("{}", res);
        let res = res.get_result();
        if let Kind::Single(res) = res {
            // We rolled one of every number, with a target number of 7 we should score a success
            // on the 7, 8, 9, and 10. So four total.
            assert_eq!(res.get_total(), 4);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn target_number_double_test() {
        let r = Solver::new("10d10 t7 tt9").unwrap();
        let res = r
            .solve_with_source(&mut MockIter { iter: &mut (1..11) })
            .unwrap();
        println!("{}", res);
        let res = res.get_result();
        if let Kind::Single(res) = res {
            // We rolled one of every number. That's a success each for the 7 and 8, and two
            // success each for the 9 and 10. So a toal of six.
            assert_eq!(res.get_total(), 6);
        } else {
            assert!(false);
        }
    }

    // Where a user has asked for a doubles threashold that is lower than the single threashold,
    // the single threashold is ignored.
    #[test]
    fn target_number_double_lower_than_target_test() {
        let r = Solver::new("10d10 tt7 t9").unwrap();
        let res = r
            .solve_with_source(&mut MockIter { iter: &mut (1..11) })
            .unwrap();
        println!("{}", res);
        let res = res.get_result();
        if let Kind::Single(res) = res {
            // We rolled one of every number. That's two successes each for the 7, 8, 9, and 10.
            // So eight total.
            assert_eq!(res.get_total(), 8);
        } else {
            assert!(false);
        }
    }

    // Where a user has asked for a doubles without singles.
    #[test]
    fn target_number_double_only() {
        let r = Solver::new("10d10 tt8").unwrap();
        let res = r
            .solve_with_source(&mut MockIter { iter: &mut (1..11) })
            .unwrap();
        println!("{}", res);
        let res = res.get_result();
        if let Kind::Single(res) = res {
            // We rolled one of every number. That's two successes each for the 8, 9, and 10.
            // So six total.
            assert_eq!(res.get_total(), 6);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn target_enum() {
        let r = Solver::new("6d6 t[2,4,6]").unwrap();
        let res = r
            .solve_with_source(&mut MockIter { iter: &mut (1..7) })
            .unwrap();
        println!("{}", res);
        let res = res.get_result();
        if let Kind::Single(res) = res {
            // We rolled one of every number. That's half of them being even
            assert_eq!(res.get_total(), 3);
        } else {
            assert!(false);
        }

        let mock = vec![1, 2, 2, 4, 6, 3];
        let res = r
            .solve_with_source(&mut MockIter {
                iter: &mut mock.into_iter(),
            })
            .unwrap();
        println!("{}", res);
        let res = res.get_result();
        if let Kind::Single(res) = res {
            // We rolled one of every number. That's half of them being even
            assert_eq!(res.get_total(), 4);
        } else {
            assert!(false);
        }

        let mock = vec![1, 3, 3, 4, 6, 3];
        let res = r
            .solve_with_source(&mut MockIter {
                iter: &mut mock.into_iter(),
            })
            .unwrap();
        println!("{}", res);
        let res = res.get_result();
        if let Kind::Single(res) = res {
            // We rolled one of every number. That's half of them being even
            assert_eq!(res.get_total(), 2);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn sandbox_test() {
        let r = Solver::new("5d6 + 4 * 2").unwrap();
        r.dices()
            .expect("Error while parsing")
            .for_each(|d| eprintln!("{}", d));

        eprintln!("{}\n{}", r.as_str(), r.solve().unwrap());
    }
}
