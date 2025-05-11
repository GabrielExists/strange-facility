use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use crate::jobs::*;

#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Resource {
    Player,
    Fish,
    FoodRation,
    Scrap,
    SpareParts,
    Submarine,
    Net,
    NetUpgraded,
    Claw,
    ClawUpgraded,
    Martha,
    MarthaAtWork,
    Forge,
}
pub fn starting_resources() -> Job {
    Job {
        button_text: "Explore the facility",
        deltas: vec![vec![
            (Resource::Player, Amount::Gain(1)),
            (Resource::FoodRation, Amount::Gain(7)),
            (Resource::Martha, Amount::Gain(1)),
            (Resource::Forge, Amount::Gain(1)),
            (Resource::Net, Amount::Gain(1)),
            (Resource::Submarine, Amount::Gain(1)),
            (Resource::Claw, Amount::Gain(1)),
        ]],
        removable: false,
        saved: false,
        instances: 1,
        id: 0,
        combination_resources: vec![],
    }
}

impl Display for Resource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Resource::Player => {
                f.write_str("Player")
            }
            Resource::Scrap => {
                f.write_str("Scrap")
            }
            Resource::SpareParts => {
                f.write_str("Spare parts")
            }
            Resource::Fish => {
                f.write_str("Fish")
            }
            Resource::FoodRation => {
                f.write_str("Food ration")
            }
            Resource::Submarine => {
                f.write_str("Sub")
            }
            Resource::Net => {
                f.write_str("Net")
            }
            Resource::NetUpgraded => {
                f.write_str("Net+")
            }
            Resource::Claw => {
                f.write_str("Claw")
            }
            Resource::ClawUpgraded => {
                f.write_str("Claw+")
            }
            Resource::Martha => {
                f.write_str("Martha")
            }
            Resource::MarthaAtWork => {
                f.write_str("WorkingMartha")
            }
            Resource::Forge => {
                f.write_str("Forge")
            }
            // Resource::ForgeGlowing => {
            //     f.write_str("Glowing forge")
            // }
        }
    }
}

pub const WIN_JOB_ID: usize = 1;

impl Resource {
    pub fn long_name(&self) -> &'static str {
        match self {
            Resource::Player => { "Player" }
            Resource::Scrap => { "Scrap metal" }
            Resource::SpareParts => { "Spare Parts" }
            Resource::Fish => { "Fish" }
            Resource::FoodRation => { "Food Ration" }
            Resource::Submarine => { "Submarine" }
            Resource::Net => { "Fishing net" }
            Resource::NetUpgraded => { "Improved fishing net" }
            Resource::Martha => { "Martha, the canning machine" }
            Resource::Forge => { "An ancient forge" }
            Resource::Claw => { "A huge mechanical claw" }
            Resource::ClawUpgraded => { "An upgraded mechanical claw" }
            // Resource::ForgeGlowing => { "A glowing forge" }
            Resource::MarthaAtWork => { "Martha, currently canning" }
        }
    }
}

pub fn attributes() -> AttributeMappings {
    BTreeMap::from([
        (Resource::Player, ResourceAttributes {
            upkeep: vec![Vec::from([(Resource::FoodRation, Amount::Spend(1))])],
            visible: false,
            display_as_name: true,
            row: 0,
        }),
        (Resource::Submarine, ResourceAttributes {
            upkeep: Vec::from([]),
            visible: true,
            display_as_name: true,
            row: 1,
        }),
        (Resource::Net, ResourceAttributes {
            upkeep: Vec::from([]),
            visible: true,
            display_as_name: true,
            row: 1,
        }),
        (Resource::NetUpgraded, ResourceAttributes {
            upkeep: Vec::from([]),
            visible: true,
            display_as_name: true,
            row: 1,
        }),
        (Resource::Claw, ResourceAttributes {
            upkeep: Vec::from([]),
            visible: true,
            display_as_name: true,
            row: 1,
        }),
        (Resource::ClawUpgraded, ResourceAttributes {
            upkeep: Vec::from([]),
            visible: true,
            display_as_name: true,
            row: 1,
        }),
        (Resource::Martha, ResourceAttributes {
            upkeep: Vec::from([]),
            visible: true,
            display_as_name: true,
            row: 1,
        }),
        (Resource::MarthaAtWork, ResourceAttributes {
            upkeep: vec![
                vec![
                    (Resource::Fish, Amount::Spend(15)),
                    (Resource::FoodRation, Amount::Gain(5)),
                ],
                vec![
                    (Resource::Martha, Amount::Gain(1)),
                    (Resource::MarthaAtWork, Amount::Spend(1)),
                ],
            ],
            visible: true,
            display_as_name: true,
            row: 1,
        }),
        (Resource::Forge, ResourceAttributes {
            upkeep: Vec::from([]),
            visible: true,
            display_as_name: true,
            row: 1,
        }),
        // (Resource::ForgeGlowing, ResourceAttributes {
        //     upkeep: Vec::from([]),
        //     visible: true,
        //     row: 1,
        // }),
    ])
}
