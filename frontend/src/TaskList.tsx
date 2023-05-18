import './TaskList.css';
import * as React from 'react';
import Task from './Task';
import RenameInput from './RenameInput';
import { Draggable, Droppable } from 'react-beautiful-dnd';
import { ITask, ITaskList } from './types';

interface Props {
    taskList: ITaskList;
    index: number;
    onTaskListRenamed: (taskList: ITaskList, newName: string) => void;
    onAddTaskToTaskList: (taskList: ITaskList) => void;
    onRemoveTaskList: (taskList: ITaskList) => void;

    onRemoveTask: (taskList: ITaskList, task: ITask) => void;
    onTaskRenamed: (taskList: ITaskList, task: ITask, newName: string) => void;
}

export default function TaskList({
    onAddTaskToTaskList,
    onRemoveTask,
    onRemoveTaskList,
    onTaskListRenamed,
    onTaskRenamed,
    index,
    taskList,
}: Props) {
    return (
        <Draggable draggableId={taskList?.id ?? ''} index={index}>
            {(provided) => (
                <div {...provided.draggableProps} ref={provided.innerRef}>
                    <div className="taskList gray-medium rounded my-2 mx-1 px-2 py-1">
                        <div
                            className="taskListHeader row mb-1 mx-0 px-1"
                            {...provided.dragHandleProps}
                            style={{ cursor: 'pointer' }}
                        >
                            <RenameInput
                                actualName={taskList.name}
                                onRenamed={(newName) => onTaskListRenamed(taskList, newName)}
                                className="taskListNameInput"
                            />
                            <button
                                onClick={() => onRemoveTaskList(taskList)}
                                type="button"
                                className="taskListRemoveButton btn btn-gray-medium col-auto ml-2"
                            >
                                <i className="fa fa-trash" />
                            </button>
                        </div>
                        <Droppable droppableId={taskList?.id ?? ''} type="TASKS">
                            {(provided) => (
                                <div className="taskListDropArea" ref={provided.innerRef} {...provided.droppableProps}>
                                    {taskList.tasks.map((task, index) => (
                                        <Task
                                            key={task.id}
                                            task={task}
                                            index={index}
                                            onRenamed={(task, newName) =>
                                                onTaskRenamed(taskList, task, newName)
                                            }
                                            onRemoveTask={(task) => onRemoveTask(taskList, task)}
                                        />
                                    ))}
                                    {provided.placeholder}
                                </div>
                            )}
                        </Droppable>
                        <button
                            onClick={() => onAddTaskToTaskList(taskList)}
                            type="button"
                            className="taskListAddTaskButton btn btn-block btn-text text-muted text-left mx-0 my-2"
                        >
                            + Add task
                        </button>
                    </div>
                </div>
            )}
        </Draggable>
    );
}
