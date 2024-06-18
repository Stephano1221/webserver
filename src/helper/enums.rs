pub enum Processing<P, F> {
    InProgress(P),
    Finished(F)
}
