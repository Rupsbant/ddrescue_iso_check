#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Clone, Copy, Hash)]
pub struct Address(pub u64);
#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Clone, Copy, Hash)]
pub struct Pass(pub u64);
#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Clone, Copy, Hash)]
pub struct Size(pub u64);

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum CurrentStatus {
    CopyNonTriedBlock,
    TrimmingBlock,
    ScrapingBlock,
    RetryBadSector,
    Filling,
    Approximate,
    Finished,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum BlockStatus {
    Untried,
    NonTrimmed,
    NonScraped,
    BadSector,
    Finished,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct MapFile {
    pub current_state: CurrentState,
    pub blocks: Vec<Block>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct CurrentState {
    pub current_pos: Address,
    pub current_status: CurrentStatus,
    pub current_pass: Option<Pass>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Block {
    pub pos: Address,
    pub size: Size,
    pub status: BlockStatus,
}
