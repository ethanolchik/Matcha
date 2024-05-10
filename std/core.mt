// FUTURE: This is future implementation for an Option[T] enum.

enum Option[T] {
    Some(T),
    None
}

func [T] (o: Option[T]) unwrap(): pub T {
    return (match o {
        Some(t) -> t,
        None -> None
    });
}

export { Option }