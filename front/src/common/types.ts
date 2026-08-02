export interface Dictionary<T> {
    [name: string]: T
}

export interface Validation {
    check: Function
    msg: string
}

export interface Item {
    value: number
    label: string
}

