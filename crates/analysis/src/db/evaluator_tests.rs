use crate::RootDatabase;
use crate::db::{self, RawF64, HubValue};
use super::evaluator::*;

#[test]
fn test_parse_literals() {
    let expr = parse_expression("42.5").unwrap();
    assert!(matches!(expr, Expr::Literal(ExprValue::Number(n)) if n == 42.5));

    let expr2 = parse_expression("'hello world'").unwrap();
    assert!(matches!(expr2, Expr::Literal(ExprValue::String(s)) if s == "hello world"));

    let expr3 = parse_expression("true").unwrap();
    assert!(matches!(expr3, Expr::Literal(ExprValue::Boolean(true))));
}

#[test]
fn test_parse_operators() {
    let expr = parse_expression("1 + 2 * 3").unwrap();
    // Multiplication should bind tighter than addition
    if let Expr::Binary { op, left, right } = expr {
        assert_eq!(op, "+");
        assert!(matches!(*left, Expr::Literal(ExprValue::Number(1.0))));
        if let Expr::Binary { op: op2, left: left2, right: right2 } = *right {
            assert_eq!(op2, "*");
            assert!(matches!(*left2, Expr::Literal(ExprValue::Number(2.0))));
            assert!(matches!(*right2, Expr::Literal(ExprValue::Number(3.0))));
        } else {
            panic!("Expected binary expression on right side");
        }
    } else {
        panic!("Expected binary expression");
    }
}

#[test]
fn test_parse_arrow_and_map() {
    let expr = parse_expression("this.companions.map(c => c.name)").unwrap();
    // Structure: Call(DotAccess(DotAccess(this, companions), map), [Arrow(c, DotAccess(c, name))])
    if let Expr::Call { target, args } = expr {
        if let Expr::DotAccess { target: target2, member } = *target {
            assert_eq!(member, "map");
            if let Expr::DotAccess { target: target3, member: member2 } = *target2 {
                assert_eq!(member2, "companions");
                assert!(matches!(*target3, Expr::Ident(ref id) if id == "this"));
            } else {
                panic!("Expected dot access");
            }
        } else {
            panic!("Expected dot access");
        }
        assert_eq!(args.len(), 1);
        if let Expr::Arrow { param, body } = &args[0] {
            assert_eq!(param, "c");
            if let Expr::DotAccess { target: target_b, member: member_b } = &**body {
                assert_eq!(member_b, "name");
                assert!(matches!(&**target_b, Expr::Ident(ref id) if id == "c"));
            } else {
                panic!("Expected dot access in arrow body");
            }
        } else {
            panic!("Expected arrow expression");
        }
    } else {
        panic!("Expected call expression");
    }
}

#[test]
fn test_evaluate_ast() {
    let mut db = RootDatabase::default();
    
    let hubgs_content = "
        DEFINITIONS [
            FIELDS [
                level: Number,
                title: Text,
                is_active: Boolean
            ],
            HUBS [
                Hero {
                    level,
                    title,
                    is_active,
                    companions -> (0..*) ALLOWS [Hero]
                }
            ]
        ],
        
        INSTANCES [
            gimli: Hero {
                level = 50.0,
                title = \"Dwarf Warrior\",
                is_active = true
            },
            
            aragorn: Hero {
                level = 80.0,
                title = \"Ranger of the North\",
                is_active = true,
                companions = [ gimli ]
            }
        ]
    ";
    
    let file = db::SourceFile::new(&mut db, "test.hubgs".to_string(), hubgs_content.to_string());
    let workspace = db::Workspace::new(&mut db, vec![file]);
    
    // Parse the hubgs file
    let result = db::parse_hubgs(&db, file);
    let instances = result.instances(&db);
    
    // Find aragorn instance
    let aragorn = instances.iter().find(|inst| inst.name(&db) == "aragorn").unwrap().clone();
    
    // Test 1: Evaluate a simple literal "10"
    let expr1 = parse_expression("10").unwrap();
    let val1 = evaluate_ast(&db, workspace, aragorn.clone(), &expr1).unwrap();
    assert_eq!(val1, HubValue::Number(RawF64::from_f64(10.0)));
    
    // Test 2: Evaluate addition "level + 5"
    let expr2 = parse_expression("level + 5").unwrap();
    let val2 = evaluate_ast(&db, workspace, aragorn.clone(), &expr2).unwrap();
    assert_eq!(val2, HubValue::Number(RawF64::from_f64(85.0)));
    
    // Test 3: Evaluate string concat "title + '!' "
    let expr3 = parse_expression("title + '!'").unwrap();
    let val3 = evaluate_ast(&db, workspace, aragorn.clone(), &expr3).unwrap();
    assert_eq!(val3, HubValue::Text("Ranger of the North!".to_string()));
    
    // Test 4: Evaluate boolean negation "!is_active"
    let expr4 = parse_expression("!is_active").unwrap();
    let val4 = evaluate_ast(&db, workspace, aragorn.clone(), &expr4).unwrap();
    assert_eq!(val4, HubValue::Boolean(false));

    // Test 5: Dot access companions length "this.companions.length"
    let expr5 = parse_expression("this.companions.length").unwrap();
    let val5 = evaluate_ast(&db, workspace, aragorn.clone(), &expr5).unwrap();
    assert_eq!(val5, HubValue::Number(RawF64::from_f64(1.0)));

    // Test 6: Array len function call "companions.len()"
    let expr6 = parse_expression("companions.len()").unwrap();
    let val6 = evaluate_ast(&db, workspace, aragorn.clone(), &expr6).unwrap();
    assert_eq!(val6, HubValue::Number(RawF64::from_f64(1.0)));

    // Test 7: Array map "companions.map(c => c.level)"
    let expr7 = parse_expression("companions.map(c => c.level)").unwrap();
    let val7 = evaluate_ast(&db, workspace, aragorn.clone(), &expr7).unwrap();
    assert_eq!(val7, HubValue::Array(vec![HubValue::Number(RawF64::from_f64(50.0))]));
}
