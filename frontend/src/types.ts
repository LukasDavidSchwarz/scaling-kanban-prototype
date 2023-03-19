export interface ITaskList {
    id: number;
    index: number;
    name: string;
    tasks: ITask[];
}

export interface ITask {
    id: string;
    name: string;
    isDone: boolean;
}

export interface ITaskListReorderEntry {
    id: number;
    newIndex: number;
}