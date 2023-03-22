export interface IBoard {
    id?: string;
    url: string;
    version: number;
    name: string;
    lists: ITaskList[];
}

export interface ITaskList {
    id?: string;
    name: string;
    tasks: ITask[];
}

export interface ITask {
    id?: string;
    name: string;
}
