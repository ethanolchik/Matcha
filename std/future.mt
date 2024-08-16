// FUTURE: This is future implementation for an Option[T] type.
// Whats not implemented:
//      - Generics
//      - Union type
//      - Match expressions
//      - Static methods

module core;

struct Option[T] {
    val: pub Union[State, T]
}

enum State {
    None
}

func [T] (Option[T]) newSome(val: T): pub Option[T] {
    return Option[T] { val: val };
}

func [T] (Option[T]) newNone(): pub Option[T] {
    return Option[T] { val: State.None };
}

func [T] (o: Option[T]) unwrap(): pub T {
    return (match o.val {
        State.None -> error("Unwrap on a None value."),
        _ -> o.val
    });
}

func [T] (o: Option[T]) isSome(): pub Bool {
    return o.val != State.None;
}

func [T] (o: Option[T]) isNone(): pub Bool {
    return o.val == State.None;
}

export { Option }