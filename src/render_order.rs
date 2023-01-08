#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub enum RenderOrder {
    Player = 0,
    // TODO(DP): check with the internet:
    //           do we need to specify values for all rest items?
    Monster = 1,
    Item = 2,
}
