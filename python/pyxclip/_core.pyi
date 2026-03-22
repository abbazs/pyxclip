__version__: str

class ClipboardError(RuntimeError): ...

def copy(
    data: None
    | str
    | tuple[int, int, bytes]
    | os.PathLike[str]
    | os.PathLike[bytes]
    | list[os.PathLike[str] | os.PathLike[bytes] | str],
    /,
) -> None: ...
def paste() -> str | dict[str, object] | list[str]: ...
def clear() -> None: ...
