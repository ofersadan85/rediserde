use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
enum Gender {
    Male,
    Female,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
enum Job {
    Unemployed,
    Employed(String),
    Owner { company: String, net_worth: i64 },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
enum DateOfBirth {
    Unknown,
    Known(u8, u8, u16),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Person {
    name: String,
    age: u32,
    gender: Gender,
    job: Job,
    date_of_birth: DateOfBirth,
    weight: Option<f64>,
}

#[test]
fn complex() {
    let alice = Person {
        name: "Alice".to_string(),
        age: 30,
        gender: Gender::Female,
        job: Job::Employed("Engineer".to_string()),
        date_of_birth: DateOfBirth::Known(15, 5, 1993),
        weight: Some(65.5),
    };
    let bob = Person {
        name: "Bob".to_string(),
        age: 25,
        gender: Gender::Male,
        job: Job::Owner {
            company: "Tech Corp".to_string(),
            net_worth: 1_000_000,
        },
        date_of_birth: DateOfBirth::Known(20, 10, 1998),
        weight: None,
    };
    let charlie = Person {
        name: "Charlie".to_string(),
        age: 40,
        gender: Gender::Male,
        job: Job::Unemployed,
        date_of_birth: DateOfBirth::Unknown,
        weight: Some(80.0),
    };
    let people = vec![alice, bob, charlie];
    let s = rediserde::to_string(&people).unwrap();
    let deserialized_people: Vec<Person> = rediserde::from_str(&s).unwrap();
    assert_eq!(deserialized_people, people);
}

#[test]
fn simple() {
    use rediserde::{from_str, to_string};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Person {
        name: String,
        age: u32,
    }

    let person = Person {
        name: "Alice".to_string(),
        age: 30,
    };
    let serialized = to_string(&person).unwrap();
    let deserialized: Person = from_str(&serialized).unwrap();
    let deserialized_raw: Person = from_str("%2\r\n+name\r\n+Alice\r\n+age\r\n:30\r\n").unwrap();
    assert_eq!(deserialized, person);
    assert_eq!(deserialized_raw, person);
}
