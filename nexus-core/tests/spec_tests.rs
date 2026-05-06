//! Nexus Spec Test Harness
//!
//! Tests organized by spec version and feature area.
//! Test IDs match nexus_test_spec.md for traceability.
//!
//! Legend:
//!   - Regular #[test]: should pass NOW (existing functionality)
//!   - #[test] #[ignore]: NOT YET IMPLEMENTED (will panic with todo!())
//!   - #[test] #[should_panic]: proves functionality DOES NOT EXIST yet

use nexus_core::*;

// ============================================================
// v0.1 — Type Registry
// ============================================================
mod registry {
    use super::*;

    #[test] // v1.reg.01
    fn define_returns_id_above_user_range() {
        let mut ctx = NexusContext::new(1);
        let id = ctx.registry_mut().define("AREA").unwrap();
        assert!(id >= 0x1000, "User-defined types must start at 0x1000+");
    }

    #[test] // v1.reg.02
    fn define_explicit_id() {
        let mut ctx = NexusContext::new(1);
        ctx.registry_mut().define_explicit("PESOS", 0x2001).unwrap();
        assert_eq!(ctx.registry().get("PESOS").unwrap(), 0x2001);
    }

    #[test] // v1.reg.03
    fn reject_duplicate_alias() {
        let mut ctx = NexusContext::new(1);
        ctx.registry_mut().define("AREA").unwrap();
        let err = ctx.registry_mut().define("AREA");
        assert!(err.is_err(), "Duplicate alias must be rejected");
    }

    #[test] // v1.reg.04
    fn get_builtin_type() {
        let ctx = NexusContext::new(1);
        assert_eq!(ctx.registry().get("METER").unwrap(), METER);
        assert_eq!(ctx.registry().get("SCALAR").unwrap(), SCALAR);
    }

    #[test] // v1.reg.05
    fn get_missing_type_errors() {
        let ctx = NexusContext::new(1);
        assert!(ctx.registry().get("NONEXISTENT").is_err());
    }

    #[test] // v1.reg.06
    fn layer0_constants_present() {
        let ctx = NexusContext::new(1);
        let r = ctx.registry();
        assert!(r.get("NULL_TYPE").is_ok());
        assert!(r.get("SCALAR").is_ok());
        assert!(r.get("METER").is_ok());
        assert!(r.get("KILOGRAM").is_ok());
        assert!(r.get("SECOND").is_ok());
        assert!(r.get("BIT").is_ok());
    }

    #[test] // v1.reg.07
    fn auto_id_increments() {
        let mut ctx = NexusContext::new(1);
        let id1 = ctx.registry_mut().define("TYPE_A").unwrap();
        let id2 = ctx.registry_mut().define("TYPE_B").unwrap();
        assert_eq!(id2, id1 + 1);
    }

    #[test] // v1.reg.08
    fn explicit_id_adjusts_auto_counter() {
        let mut ctx = NexusContext::new(1);
        ctx.registry_mut().define_explicit("HIGH", 0x5000).unwrap();
        let next = ctx.registry_mut().define("AFTER_HIGH").unwrap();
        assert_eq!(next, 0x5001);
    }
}

// ============================================================
// v0.1 — Stack Operations
// ============================================================
mod stack {
    use super::*;

    #[test] // v1.stk.01 (exists as test_push_pop, re-verified here)
    fn push_pop_scalar_roundtrip() {
        let mut ctx = NexusContext::new(1);
        ctx.push_scalar(42.0, SCALAR);
        let val = ctx.pop().unwrap();
        assert_eq!(val.data, vec![42.0]);
        assert_eq!(val.ontic_type, SCALAR);
    }

    #[test] // v1.stk.02
    fn pop_empty_stack_errors() {
        let mut ctx = NexusContext::new(1);
        assert!(ctx.pop().is_err());
    }

    #[test] // v1.stk.03
    fn stack_is_lifo() {
        let mut ctx = NexusContext::new(1);
        ctx.push_scalar(1.0, METER);
        ctx.push_scalar(2.0, SECOND);
        let top = ctx.pop().unwrap();
        assert_eq!(top.ontic_type, SECOND, "LIFO: last pushed should pop first");
        let next = ctx.pop().unwrap();
        assert_eq!(next.ontic_type, METER);
    }

    #[test] // v1.stk.04
    fn multiple_items_depth() {
        let mut ctx = NexusContext::new(1);
        ctx.push_scalar(1.0, METER);
        ctx.push_scalar(2.0, METER);
        ctx.push_scalar(3.0, METER);
        assert_eq!(ctx.stack_depth(), 3);
    }

    #[test] // peek without consuming
    fn peek_returns_top_without_consuming() {
        let mut ctx = NexusContext::new(1);
        ctx.push_scalar(42.0, METER);
        let peeked = ctx.peek().unwrap();
        assert_eq!(peeked.ontic_type, METER);
        assert_eq!(ctx.stack_depth(), 1, "peek must not consume");
    }

    #[test]
    fn peek_empty_stack_errors() {
        let ctx = NexusContext::new(1);
        assert!(ctx.peek().is_err());
    }
}

// ============================================================
// v0.1 — Apply (Dyadic Operations)
// ============================================================
mod apply {
    use super::*;

    #[test] // v1.app.01
    fn add_two_scalars() {
        let mut ctx = NexusContext::new(1);
        ctx.push_scalar(3.0, METER);
        ctx.push_scalar(4.0, METER);
        ctx.apply(Op::Add, &[METER, METER], &[METER]).unwrap();
        let res = ctx.pop().unwrap();
        assert_eq!(res.scalar_value(), 7.0);
        assert_eq!(res.ontic_type, METER);
    }

    #[test] // v1.app.02
    fn multiply_two_scalars() {
        let mut ctx = NexusContext::new(1);
        let area = 0x1010;
        ctx.push_scalar(3.0, METER);
        ctx.push_scalar(3.0, METER);
        ctx.apply(Op::Multiply, &[METER, METER], &[area]).unwrap();
        let res = ctx.pop().unwrap();
        assert_eq!(res.scalar_value(), 9.0);
        assert_eq!(res.ontic_type, area);
    }

    #[test] // v1.app.03
    fn divide_by_zero_errors() {
        let mut ctx = NexusContext::new(1);
        ctx.push_scalar(1.0, SCALAR);
        ctx.push_scalar(0.0, SCALAR);
        let result = ctx.apply(Op::Divide, &[SCALAR, SCALAR], &[SCALAR]);
        assert!(result.is_err(), "Division by zero must error");
    }

    #[test] // v1.app.04
    fn type_mismatch_errors() {
        let mut ctx = NexusContext::new(1);
        ctx.push_scalar(1.0, METER);
        ctx.push_scalar(1.0, METER);
        let result = ctx.apply(Op::Add, &[SECOND, SECOND], &[SECOND]);
        assert!(result.is_err(), "Type mismatch must error");
    }

    #[test] // v1.app.05
    fn stack_underflow_on_apply() {
        let mut ctx = NexusContext::new(1);
        ctx.push_scalar(1.0, METER);
        let result = ctx.apply(Op::Add, &[METER, METER], &[METER]);
        assert!(result.is_err(), "Not enough values must underflow");
    }

    #[test] // all six verbs work on scalars
    fn all_pervasive_verbs_scalar() {
        let ops = [
            (Op::Add, 3.0, 2.0, 5.0),
            (Op::Subtract, 5.0, 3.0, 2.0),
            (Op::Multiply, 4.0, 3.0, 12.0),
            (Op::Divide, 10.0, 2.0, 5.0),
            (Op::Max, 3.0, 7.0, 7.0),
            (Op::Min, 3.0, 7.0, 3.0),
        ];
        for (op, a, b, expected) in ops {
            let mut ctx = NexusContext::new(1);
            ctx.push_scalar(a, SCALAR);
            ctx.push_scalar(b, SCALAR);
            ctx.apply(op, &[SCALAR, SCALAR], &[SCALAR]).unwrap();
            let res = ctx.pop().unwrap();
            assert_eq!(res.scalar_value(), expected, "Failed for {:?}", op);
        }
    }
}

// ============================================================
// v0.1 — Consistency Ledger
// ============================================================
mod ledger {
    use super::*;

    #[test] // v1.led.01
    fn novel_signature_recorded() {
        let mut ledger = ConsistencyLedger::new();
        let v = ledger.check(Op::Multiply, &[METER, METER], &[0x1010]);
        assert!(matches!(v, LedgerVerdict::Novel));
    }

    #[test] // v1.led.02
    fn consistent_reuse() {
        let mut ledger = ConsistencyLedger::new();
        ledger.check(Op::Multiply, &[METER, METER], &[0x1010]);
        let v = ledger.check(Op::Multiply, &[METER, METER], &[0x1010]);
        assert!(matches!(v, LedgerVerdict::Consistent));
    }

    #[test] // v1.led.03
    fn contradiction_detected() {
        let mut ledger = ConsistencyLedger::new();
        ledger.check(Op::Multiply, &[METER, METER], &[0x1010]); // AREA
        let v = ledger.check(Op::Multiply, &[METER, METER], &[0x9999]); // BIRDS
        assert!(matches!(v, LedgerVerdict::Contradiction(_)));
    }

    #[test] // v1.led.04
    fn contradiction_includes_prior_signature() {
        let mut ledger = ConsistencyLedger::new();
        ledger.check(Op::Multiply, &[METER, METER], &[0x1010]);
        let v = ledger.check(Op::Multiply, &[METER, METER], &[0x9999]);
        if let LedgerVerdict::Contradiction(prior) = v {
            assert_eq!(prior.outputs, vec![0x1010], "Prior sig must contain original output");
        } else {
            panic!("Expected Contradiction");
        }
    }

    #[test] // v1.led.05
    fn different_ops_dont_interfere() {
        let mut ledger = ConsistencyLedger::new();
        let v1 = ledger.check(Op::Add, &[METER, METER], &[METER]);
        let v2 = ledger.check(Op::Multiply, &[METER, METER], &[0x1010]);
        assert!(matches!(v1, LedgerVerdict::Novel));
        assert!(matches!(v2, LedgerVerdict::Novel));
    }

    #[test] // Ledger integrates with apply
    fn apply_returns_ledger_verdict() {
        let mut ctx = NexusContext::new(1);
        let area = 0x1010;

        ctx.push_scalar(2.0, METER);
        ctx.push_scalar(3.0, METER);
        let v1 = ctx.apply(Op::Multiply, &[METER, METER], &[area]).unwrap();
        assert!(matches!(v1, LedgerVerdict::Novel));

        ctx.push_scalar(4.0, METER);
        ctx.push_scalar(5.0, METER);
        let v2 = ctx.apply(Op::Multiply, &[METER, METER], &[area]).unwrap();
        assert!(matches!(v2, LedgerVerdict::Consistent));
    }
}

// ============================================================
// v0.1 — Type Bridges
// ============================================================
mod bridges {
    use super::*;

    #[test] // v1.brg.01
    fn bridge_converts_scalar() {
        let mut ctx = NexusContext::new(1);
        let seconds = ctx.registry_mut().define("SECONDS").unwrap();
        let minutes = ctx.registry_mut().define("MINUTES").unwrap();
        ctx.registry_mut().bridge(seconds, minutes, Op::Divide, 60.0);

        ctx.push_scalar(3600.0, seconds);
        ctx.convert_to(minutes).unwrap();
        let res = ctx.pop().unwrap();
        assert_eq!(res.scalar_value(), 60.0);
        assert_eq!(res.ontic_type, minutes);
    }

    #[test] // v1.brg.02
    fn convert_same_type_is_noop() {
        let mut ctx = NexusContext::new(1);
        ctx.push_scalar(42.0, METER);
        ctx.convert_to(METER).unwrap();
        let res = ctx.pop().unwrap();
        assert_eq!(res.scalar_value(), 42.0);
        assert_eq!(res.ontic_type, METER);
    }

    #[test] // v1.brg.03
    fn missing_bridge_errors() {
        let mut ctx = NexusContext::new(1);
        ctx.push_scalar(42.0, METER);
        let result = ctx.convert_to(SECOND);
        assert!(result.is_err());
        // Value should be preserved on stack after failed conversion
        let res = ctx.pop().unwrap();
        assert_eq!(res.scalar_value(), 42.0);
        assert_eq!(res.ontic_type, METER);
    }

    #[test] // v1.brg.04
    fn bridge_converts_all_tensor_elements() {
        let mut ctx = NexusContext::new(1);
        let cm = ctx.registry_mut().define("CM").unwrap();
        ctx.registry_mut().bridge(METER, cm, Op::Multiply, 100.0);

        ctx.push_tensor(vec![1.0, 2.0, 3.0], vec![3], METER);
        ctx.convert_to(cm).unwrap();
        let res = ctx.pop().unwrap();
        assert_eq!(res.data, vec![100.0, 200.0, 300.0]);
        assert_eq!(res.ontic_type, cm);
    }
}

// ============================================================
// v0.2 — Tensor Operations
// ============================================================
mod tensors {
    use super::*;

    #[test] // v2.ten.03 — reversed broadcast order
    fn tensor_scalar_broadcast_reversed() {
        let mut ctx = NexusContext::new(1);
        let result_type = 0x1020;
        ctx.push_scalar(2.0, SCALAR);
        ctx.push_tensor(vec![1.0, 2.0, 3.0], vec![3], METER);
        ctx.apply(Op::Multiply, &[SCALAR, METER], &[result_type]).unwrap();
        let res = ctx.pop().unwrap();
        assert_eq!(res.data, vec![2.0, 4.0, 6.0]);
    }

    #[test] // v2.ten.04
    fn shape_mismatch_errors() {
        let mut ctx = NexusContext::new(1);
        ctx.push_tensor(vec![1.0, 2.0, 3.0], vec![3], METER);
        ctx.push_tensor(vec![1.0, 2.0], vec![2], METER);
        let result = ctx.apply(Op::Add, &[METER, METER], &[METER]);
        assert!(result.is_err());
    }

    #[test] // v2.ten.05 — all six verbs on vectors
    fn all_pervasive_verbs_vector() {
        let a = vec![10.0, 20.0, 30.0];
        let b = vec![3.0, 5.0, 10.0];
        let cases: Vec<(Op, Vec<f64>)> = vec![
            (Op::Add, vec![13.0, 25.0, 40.0]),
            (Op::Subtract, vec![7.0, 15.0, 20.0]),
            (Op::Multiply, vec![30.0, 100.0, 300.0]),
            (Op::Divide, vec![10.0/3.0, 4.0, 3.0]),
            (Op::Max, vec![10.0, 20.0, 30.0]),
            (Op::Min, vec![3.0, 5.0, 10.0]),
        ];
        for (op, expected) in cases {
            let mut ctx = NexusContext::new(1);
            ctx.push_tensor(a.clone(), vec![3], SCALAR);
            ctx.push_tensor(b.clone(), vec![3], SCALAR);
            ctx.apply(op, &[SCALAR, SCALAR], &[SCALAR]).unwrap();
            let res = ctx.pop().unwrap();
            assert_eq!(res.data, expected, "Failed for {:?}", op);
        }
    }
}

// ============================================================
// v0.2 — Adverbs
// ============================================================
mod adverbs {
    use super::*;

    #[test] // v2.adv.02 — Reduce(Multiply)
    fn reduce_multiply() {
        let mut ctx = NexusContext::new(1);
        let volume = 0x1030;
        ctx.push_tensor(vec![2.0, 3.0, 4.0], vec![3], METER);
        ctx.apply_adverb(Adverb::Reduce, Op::Multiply, &[METER], &[volume]).unwrap();
        let res = ctx.pop().unwrap();
        assert_eq!(res.scalar_value(), 24.0);
    }

    #[test] // v2.adv.03 — Reduce on empty
    fn reduce_empty_errors() {
        let mut ctx = NexusContext::new(1);
        ctx.push_tensor(vec![], vec![0], METER);
        let result = ctx.apply_adverb(Adverb::Reduce, Op::Add, &[METER], &[METER]);
        assert!(result.is_err());
    }

    // ---- NOT YET IMPLEMENTED ----

    #[test]
    #[should_panic(expected = "not fully implemented")]
    // v2.adv.04 — Scan(Add)
    fn scan_add_not_implemented() {
        let mut ctx = NexusContext::new(1);
        ctx.push_tensor(vec![1.0, 2.0, 3.0], vec![3], SECOND);
        // When implemented: should produce [1, 3, 6] :: SECOND
        ctx.apply_adverb(Adverb::Scan, Op::Add, &[SECOND], &[SECOND]).unwrap();
    }

    #[test]
    #[should_panic(expected = "not fully implemented")]
    // v2.adv.05 — Each
    fn each_not_implemented() {
        let mut ctx = NexusContext::new(1);
        ctx.push_tensor(vec![4.0, 9.0, 16.0], vec![3], METER);
        ctx.apply_adverb(Adverb::Each, Op::Add, &[METER], &[METER]).unwrap();
    }

    #[test]
    #[should_panic(expected = "not fully implemented")]
    // v2.adv.06 — Table (Outer Product)
    fn table_not_implemented() {
        let mut ctx = NexusContext::new(1);
        let area = 0x1010;
        ctx.push_tensor(vec![2.0, 3.0], vec![2], METER);
        ctx.push_tensor(vec![4.0, 5.0], vec![2], METER);
        // When implemented: should produce [[8,10],[12,15]] :: AREA
        ctx.apply_adverb(Adverb::Table, Op::Multiply, &[METER, METER], &[area]).unwrap();
    }
}

// ============================================================
// v0.2 — Stack Manipulation (Pick)
// ============================================================
mod pick {
    use super::*;

    #[test]
    // v2.stk.01 — dup via pick
    fn pick_dup() {
        let mut ctx = NexusContext::new(1);
        ctx.push_scalar(42.0, METER);
        ctx.pick(&[0, 0]).unwrap(); // dup
        assert_eq!(ctx.stack_depth(), 2);
        let a = ctx.pop().unwrap();
        let b = ctx.pop().unwrap();
        assert_eq!(a.scalar_value(), 42.0);
        assert_eq!(b.scalar_value(), 42.0);
    }

    #[test]
    // v2.stk.02 — swap via pick
    fn pick_swap() {
        let mut ctx = NexusContext::new(1);
        ctx.push_scalar(1.0, METER);   // bottom
        ctx.push_scalar(2.0, SECOND);  // top
        ctx.pick(&[1, 0]).unwrap();    // swap
        let top = ctx.pop().unwrap();
        let bot = ctx.pop().unwrap();
        assert_eq!(top.ontic_type, METER, "After swap, METER should be on top");
        assert_eq!(bot.ontic_type, SECOND);
    }

    #[test]
    // v2.stk.03 — over via pick
    fn pick_over() {
        let mut ctx = NexusContext::new(1);
        ctx.push_scalar(1.0, METER);   // bottom (index 0 after drain)
        ctx.push_scalar(2.0, SECOND);  // top (index 1 after drain)
        ctx.pick(&[0, 1, 0]).unwrap(); // over: [second, top, second]
        assert_eq!(ctx.stack_depth(), 3);
        let top = ctx.pop().unwrap();
        assert_eq!(top.ontic_type, METER, "Over copies second-from-top to top");
    }

    #[test]
    // v2.stk.05 — drop via drop_top
    fn drop_top() {
        let mut ctx = NexusContext::new(1);
        ctx.push_scalar(1.0, METER);
        ctx.push_scalar(2.0, SECOND);
        ctx.drop_top().unwrap();
        assert_eq!(ctx.stack_depth(), 1);
        let remaining = ctx.pop().unwrap();
        assert_eq!(remaining.ontic_type, METER);
    }

    #[test]
    fn pick_empty_is_noop() {
        let mut ctx = NexusContext::new(1);
        ctx.push_scalar(1.0, METER);
        ctx.pick(&[]).unwrap();
        assert_eq!(ctx.stack_depth(), 1);
    }

    #[test]
    fn pick_underflow_errors() {
        let mut ctx = NexusContext::new(1);
        ctx.push_scalar(1.0, METER);
        let result = ctx.pick(&[1, 0]); // needs 2 items, only have 1
        assert!(result.is_err());
    }

    #[test]
    fn drop_empty_errors() {
        let mut ctx = NexusContext::new(1);
        assert!(ctx.drop_top().is_err());
    }
}

// ============================================================
// v0.1 — Goals (implemented — should pass NOW)
// ============================================================
mod goals {
    use super::*;

    #[test]
    fn goal_set_and_read() {
        let mut ctx = NexusContext::new(1);
        assert!(ctx.current_goal().is_none());
        ctx.goal("Compute geometric mean");
        assert_eq!(ctx.current_goal(), Some("Compute geometric mean"));
    }

    #[test]
    fn goal_done_clears() {
        let mut ctx = NexusContext::new(1);
        ctx.goal("Compute geometric mean");
        ctx.goal_done();
        assert!(ctx.current_goal().is_none());
    }
}

// ============================================================
// v0.3 — Serialization
// ============================================================
mod serialization {
    use super::*;

    #[test]
    // v3.ser.01 — serialize empty context
    fn serialize_empty() {
        let ctx = NexusContext::new(1);
        let json = ctx.serialize().unwrap();
        assert!(json.contains("agent_id"));
        assert!(json.contains("registry"));
        assert!(json.contains("stack"));
    }

    #[test]
    // v3.ser.02 — round-trip empty context
    fn roundtrip_empty() {
        let ctx = NexusContext::new(42);
        let json = ctx.serialize().unwrap();
        let ctx2 = NexusContext::deserialize(&json).unwrap();
        assert_eq!(ctx2.agent_id, 42);
        assert_eq!(ctx2.stack_depth(), 0);
    }

    #[test]
    // v3.ser.03 — round-trip with registry
    fn roundtrip_registry() {
        let mut ctx = NexusContext::new(1);
        let area = ctx.registry_mut().define("AREA").unwrap();
        let json = ctx.serialize().unwrap();
        let ctx2 = NexusContext::deserialize(&json).unwrap();
        assert_eq!(ctx2.registry().get("AREA").unwrap(), area);
        assert_eq!(ctx2.registry().get("METER").unwrap(), METER);
    }

    #[test]
    // v3.ser.04 — round-trip with ledger
    fn roundtrip_ledger() {
        let mut ctx = NexusContext::new(1);
        let area = 0x1010;
        ctx.push_scalar(3.0, METER);
        ctx.push_scalar(3.0, METER);
        ctx.apply(Op::Multiply, &[METER, METER], &[area]).unwrap();
        ctx.pop().unwrap(); // consume result

        let json = ctx.serialize().unwrap();
        let mut ctx2 = NexusContext::deserialize(&json).unwrap();

        // The ledger should remember Multiply([M,M]) → [AREA]
        ctx2.push_scalar(2.0, METER);
        ctx2.push_scalar(2.0, METER);
        let verdict = ctx2.apply(Op::Multiply, &[METER, METER], &[area]).unwrap();
        assert!(matches!(verdict, LedgerVerdict::Consistent));
    }

    #[test]
    // v3.ser.05 — round-trip with stack
    fn roundtrip_stack() {
        let mut ctx = NexusContext::new(1);
        ctx.push_tensor(vec![1.0, 2.0, 3.0], vec![3], METER);
        ctx.push_scalar(42.0, SECOND);

        let json = ctx.serialize().unwrap();
        let mut ctx2 = NexusContext::deserialize(&json).unwrap();

        assert_eq!(ctx2.stack_depth(), 2);
        let top = ctx2.pop().unwrap();
        assert_eq!(top.scalar_value(), 42.0);
        assert_eq!(top.ontic_type, SECOND);
        let next = ctx2.pop().unwrap();
        assert_eq!(next.data, vec![1.0, 2.0, 3.0]);
        assert_eq!(next.ontic_type, METER);
    }

    #[test]
    // v3.ser.06 — round-trip with bridges
    fn roundtrip_bridges() {
        let mut ctx = NexusContext::new(1);
        let cm = ctx.registry_mut().define("CM").unwrap();
        ctx.registry_mut().bridge(METER, cm, Op::Multiply, 100.0);

        let json = ctx.serialize().unwrap();
        let mut ctx2 = NexusContext::deserialize(&json).unwrap();

        ctx2.push_scalar(2.5, METER);
        ctx2.convert_to(cm).unwrap();
        let res = ctx2.pop().unwrap();
        assert_eq!(res.scalar_value(), 250.0);
        assert_eq!(res.ontic_type, cm);
    }

    #[test]
    // v3.ser.07 — JSON is human-readable (contains type names)
    fn json_contains_type_names() {
        let mut ctx = NexusContext::new(1);
        ctx.registry_mut().define("AREA").unwrap();
        let json = ctx.serialize().unwrap();
        assert!(json.contains("AREA"), "JSON should contain type alias names");
        assert!(json.contains("METER"), "JSON should contain built-in names");
    }

    #[test]
    // goals survive serialization
    fn roundtrip_goals() {
        let mut ctx = NexusContext::new(1);
        ctx.goal("Compute geometric mean");
        let json = ctx.serialize().unwrap();
        let ctx2 = NexusContext::deserialize(&json).unwrap();
        assert_eq!(ctx2.current_goal(), Some("Compute geometric mean"));
    }

    #[test]
    // ledger detects contradiction after deserialization
    fn roundtrip_ledger_contradiction() {
        let mut ctx = NexusContext::new(1);
        let area = 0x1010;
        ctx.push_scalar(3.0, METER);
        ctx.push_scalar(3.0, METER);
        ctx.apply(Op::Multiply, &[METER, METER], &[area]).unwrap();
        ctx.pop().unwrap();

        let json = ctx.serialize().unwrap();
        let mut ctx2 = NexusContext::deserialize(&json).unwrap();

        // Try a contradictory declaration
        let velocity = 0x2020;
        ctx2.push_scalar(2.0, METER);
        ctx2.push_scalar(2.0, METER);
        let verdict = ctx2.apply(Op::Multiply, &[METER, METER], &[velocity]).unwrap();
        assert!(matches!(verdict, LedgerVerdict::Contradiction(_)),
            "Deserialized ledger must detect contradictions from prior session");
    }
}

// ============================================================
// v0.3 — Assert-Gated Effects & Rollback
// ============================================================
mod assertions {
    use super::*;

    #[test]
    // v3.asr.01 — assert_consistent passes after novel operation
    fn assert_consistent_passes_on_novel() {
        let mut ctx = NexusContext::new(1);
        let area = 0x1010;
        ctx.push_scalar(3.0, METER);
        ctx.push_scalar(3.0, METER);
        ctx.apply(Op::Multiply, &[METER, METER], &[area]).unwrap();
        ctx.assert_consistent().unwrap(); // Novel → should pass
    }

    #[test]
    // v3.asr.02 — assert_consistent fails after contradiction
    fn assert_consistent_fails_on_contradiction() {
        let mut ctx = NexusContext::new(1);
        let area = 0x1010;
        let velocity = 0x2020;

        ctx.push_scalar(3.0, METER);
        ctx.push_scalar(3.0, METER);
        ctx.apply(Op::Multiply, &[METER, METER], &[area]).unwrap();
        ctx.assert_consistent().unwrap(); // pass → creates savepoint

        ctx.push_scalar(2.0, METER);
        ctx.push_scalar(2.0, METER);
        ctx.apply(Op::Multiply, &[METER, METER], &[velocity]).unwrap(); // contradiction!

        let result = ctx.assert_consistent();
        assert!(result.is_err(), "Should fail after contradiction");
    }

    #[test]
    // v3.asr.03 — rollback restores stack after failed assert
    fn rollback_restores_stack() {
        let mut ctx = NexusContext::new(1);
        let area = 0x1010;
        let velocity = 0x2020;

        ctx.push_scalar(3.0, METER);
        ctx.push_scalar(3.0, METER);
        ctx.apply(Op::Multiply, &[METER, METER], &[area]).unwrap();
        ctx.assert_consistent().unwrap(); // savepoint with [9.0::AREA] on stack

        let depth_at_savepoint = ctx.stack_depth(); // 1 (the result)

        // Do more work that contradicts
        ctx.push_scalar(2.0, METER);
        ctx.push_scalar(2.0, METER);
        ctx.apply(Op::Multiply, &[METER, METER], &[velocity]).unwrap();
        assert_eq!(ctx.stack_depth(), 2); // result + contradictory result

        let _ = ctx.assert_consistent(); // fails → rollback
        assert_eq!(ctx.stack_depth(), depth_at_savepoint,
            "Stack must be restored to savepoint state after failed assert");
    }

    #[test]
    // v3.asr.04 — rollback restores ledger (contradiction is undone)
    fn rollback_restores_ledger() {
        let mut ctx = NexusContext::new(1);
        let area = 0x1010;
        let velocity = 0x2020;

        ctx.push_scalar(3.0, METER);
        ctx.push_scalar(3.0, METER);
        ctx.apply(Op::Multiply, &[METER, METER], &[area]).unwrap();
        ctx.assert_consistent().unwrap(); // savepoint

        // Contradict, then rollback
        ctx.push_scalar(2.0, METER);
        ctx.push_scalar(2.0, METER);
        ctx.apply(Op::Multiply, &[METER, METER], &[velocity]).unwrap();
        let _ = ctx.assert_consistent(); // fails → rollback

        // After rollback, the contradictory entry should be gone
        // So re-doing the CORRECT operation should be Consistent, not Contradiction
        ctx.push_scalar(5.0, METER);
        ctx.push_scalar(5.0, METER);
        let verdict = ctx.apply(Op::Multiply, &[METER, METER], &[area]).unwrap();
        assert!(matches!(verdict, LedgerVerdict::Consistent),
            "After rollback, the original signature should still be in the ledger");
    }

    #[test]
    // v3.asr.05
    fn assert_type_passes_when_correct() {
        let mut ctx = NexusContext::new(1);
        ctx.push_scalar(42.0, METER);
        ctx.assert_type(METER).unwrap();
        assert_eq!(ctx.stack_depth(), 1, "assert_type should not consume the value");
    }

    #[test]
    // v3.asr.06
    fn assert_type_fails_and_rolls_back() {
        let mut ctx = NexusContext::new(1);
        ctx.push_scalar(1.0, METER);
        let _ = ctx.savepoint();

        ctx.push_scalar(2.0, SECOND); // push wrong type
        let result = ctx.assert_type(METER);
        assert!(result.is_err());

        // Should have rolled back — SECOND push is undone
        assert_eq!(ctx.stack_depth(), 1);
        let top = ctx.peek().unwrap();
        assert_eq!(top.ontic_type, METER);
    }

    #[test]
    // v3.asr.07
    fn assert_shape_passes_when_correct() {
        let mut ctx = NexusContext::new(1);
        ctx.push_tensor(vec![1.0, 2.0, 3.0], vec![3], METER);
        ctx.assert_shape(&[3]).unwrap();
    }

    #[test]
    // v3.asr.08 — nested savepoints
    fn nested_savepoints() {
        let mut ctx = NexusContext::new(1);
        ctx.push_scalar(1.0, METER);
        ctx.assert_consistent().unwrap(); // S1: stack=[1.0::M]

        ctx.push_scalar(2.0, METER);
        ctx.assert_consistent().unwrap(); // S2: stack=[1.0::M, 2.0::M]

        ctx.push_scalar(3.0, METER);
        // Fail: should rollback to S2, not S1
        let result = ctx.assert_type(SECOND); // 3.0 is METER, not SECOND
        assert!(result.is_err());

        assert_eq!(ctx.stack_depth(), 2, "Should rollback to S2 (2 items), not S1 (1 item)");
    }

    #[test]
    // v3.asr.10 — manual savepoint/rollback
    fn manual_savepoint_rollback() {
        let mut ctx = NexusContext::new(1);
        ctx.push_scalar(1.0, METER);
        let sp = ctx.savepoint();
        ctx.push_scalar(2.0, SECOND);
        ctx.push_scalar(3.0, KILOGRAM);
        assert_eq!(ctx.stack_depth(), 3);

        ctx.rollback(sp);
        assert_eq!(ctx.stack_depth(), 1, "Rollback should restore to savepoint state");
        let top = ctx.pop().unwrap();
        assert_eq!(top.ontic_type, METER);
    }
}

// ============================================================
// Integration: End-to-End Workflows
// ============================================================
mod workflows {
    use super::*;

    #[test]
    /// The geometric mean example from v0.2 spec §4
    fn geometric_mean_pipeline() {
        let mut ctx = NexusContext::new(1);
        let area = ctx.registry_mut().define("AREA").unwrap();

        // Push two lengths, reduce-multiply to get area
        ctx.push_tensor(vec![4.0, 9.0], vec![2], METER);
        ctx.apply_adverb(Adverb::Reduce, Op::Multiply, &[METER], &[area]).unwrap();

        let res = ctx.pop().unwrap();
        assert_eq!(res.scalar_value(), 36.0);
        assert_eq!(res.ontic_type, area);
    }

    #[test]
    /// Verify the ledger catches contradictions through the full apply path
    fn full_contradiction_detection() {
        let mut ctx = NexusContext::new(1);
        let area = 0x1010;
        let velocity = 0x2020;

        // First use: METER * METER → AREA (novel)
        ctx.push_scalar(3.0, METER);
        ctx.push_scalar(3.0, METER);
        let v1 = ctx.apply(Op::Multiply, &[METER, METER], &[area]).unwrap();
        assert!(matches!(v1, LedgerVerdict::Novel));
        ctx.pop().unwrap(); // consume result

        // Second use: METER * METER → VELOCITY (contradiction!)
        ctx.push_scalar(3.0, METER);
        ctx.push_scalar(3.0, METER);
        let v2 = ctx.apply(Op::Multiply, &[METER, METER], &[velocity]).unwrap();
        assert!(matches!(v2, LedgerVerdict::Contradiction(_)));
    }
}
