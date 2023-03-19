import React, {useEffect} from 'react';
import './App.css';
import TaskListGenerator from "./TaskListGenerator";
import produce from 'immer';
import TaskList from "./TaskList";
import axios from "axios";
import useStateCallback from "./useStateCallback";
import jsonpatch from "fast-json-patch";
import {DragDropContext, DraggableLocation, Droppable, DropResult} from 'react-beautiful-dnd';
import styled from "styled-components";
import {ITask, ITaskList, ITaskListReorderEntry} from "./types";
import useWebSocket from "react-use-websocket";


interface State {
    taskLists: ITaskList[];
}

//TODO: use environment variable to determine api endpoint
const TASK_LIST_ENDPOINT = './api/taskLists';

const TASK_LIST_DROPPABLE_TYPE = 'TASK_LIST';

const TaskListContainer = styled.div`
  display: flex;
`;

export default function App () {
    const [state, setState] = useStateCallback<State>({
        taskLists: new Array<ITaskList>()
    });
    
    const { sendJsonMessage, lastJsonMessage, readyState } = useWebSocket(
        "ws://localhost:3512/board/0/ws",
        {
            onOpen: () => console.log("Websocket opened"),
            onMessage: message => console.log(`Received ${message.data}`),
            shouldReconnect: (closeEvent) => true,
        });
    

    // eslint-disable-next-line react-hooks/exhaustive-deps
    useEffect(updateState, []) // empty dependency array so that hook is only called when component is mounted/unmounted

    if (lastJsonMessage != null && lastJsonMessage !== state) {
        console.log("new message:", lastJsonMessage);
        setState(lastJsonMessage);
    }

    function getTaskListViaId(taskListId: string) {
        return state.taskLists.find(taskList => taskList.id.toString() === taskListId);
    }

    function handleCreateNewTaskList (name = "New ToDo List") {
        let newTaskList: ITaskList = {
            id: 0,
            index: 0,
            name: name,
            tasks: []
        };

        let newState = produce(state, draft => {
            draft.taskLists.push(newTaskList);
        });

        if (readyState === 1) {
            sendJsonMessage(newState);
        }
        
        axios.post(TASK_LIST_ENDPOINT, newTaskList)
            .then(updateState)
            .catch(error => {
                console.error(`Failed to create task list ${newTaskList}`);
                console.error(error);
            });
    }

    function handleAddTaskToTaskList (taskList: ITaskList, name = "New Task", isDone = false): void {
        let newTask: ITask = {
            id: '',
            name,
            isDone
        };

        axios.post(`${TASK_LIST_ENDPOINT}/${taskList.id}/tasks`, newTask)
            .then(updateState)
            .catch(error => {
                console.error(`Failed add task ${newTask} to task list ${taskList}!`);
                console.error(error);
            });
    }

    function handleRemoveTaskList (taskList: ITaskList): void {
        axios.delete(`${TASK_LIST_ENDPOINT}/${taskList.id}`)
            .then(updateState)
            .catch(error => {
                console.error(`Failed to delete task list ${taskList}!`);
                console.error(error);
            });
    }

    function handleRemoveTask(taskList: ITaskList, task: ITask): void {
        let taskIndex = taskList.tasks.indexOf(task);

        let updatedTaskList = produce(taskList, draft => {
            draft.tasks.splice(taskIndex, 1);
        });
        updateBoard(taskList, updatedTaskList);
    }

    function handleTaskListRenamed(taskList: ITaskList, newName: string): void {
        let updatedTaskList = produce(taskList, draft => {
            draft.name = newName;
        });
        updateBoard(taskList, updatedTaskList);
    }

    function handleTaskRenamed(taskList: ITaskList, task: ITask, newName: string): void {
        let taskIndex = taskList.tasks.indexOf(task);

        let updatedTaskList = produce(taskList, draft => {
            draft.tasks[taskIndex].name = newName;
        });

        updateBoard(taskList, updatedTaskList);
    }

    function handleTaskIsDoneChanged(taskList: ITaskList, task: ITask, newVal: boolean): void {
        let taskIndex = taskList.tasks.indexOf(task);

        let updatedTaskList = produce(taskList, draft => {
            draft.tasks[taskIndex].isDone = newVal;
        });
        updateBoard(taskList, updatedTaskList);
    }

    function handleTaskMovedInSameTaskList(taskList: ITaskList, source: DraggableLocation, destination: DraggableLocation) {
        let updatedTaskList = produce(taskList, draft => {
            let [removedTask] = draft.tasks.splice(source.index, 1);
            draft.tasks.splice(destination.index, 0, removedTask);
        });

        updateBoard(taskList, updatedTaskList);
    }

    function handleTaskMovedToSeparateTaskList(sourceTaskList: ITaskList, destinationTaskList: ITaskList, task: ITask, source: DraggableLocation, destination: DraggableLocation) {
        let updatedSourceTaskList = produce(sourceTaskList, draft => {
            draft.tasks.splice(source.index, 1);
        });

        let updatedDestinationTaskList = produce(destinationTaskList, draft => {
            draft.tasks.splice(destination.index, 0, task);
        });

        updateBoard(sourceTaskList, updatedSourceTaskList);
        updateBoard(destinationTaskList, updatedDestinationTaskList);
    }

    function handleTaskListMoved(source: DraggableLocation, destination: DraggableLocation) {
        let taskListReorderEntries = new Array<ITaskListReorderEntry>();
        let stateTransformer = produce<State>(draft => {
            let [movedTask] = draft.taskLists.splice(source.index, 1);
            draft.taskLists.splice(destination.index, 0, movedTask);

            // update index variables to reflect changes
            draft.taskLists.forEach((taskList, index) => {
                if (taskList.index !== index) {
                    taskList.index = index;
                    taskListReorderEntries.push({
                        id: taskList.id,
                        newIndex: index,
                    });
                }
            });
        });

        // execute state transformer once, to populate the 'taskListReorderEntries' Array so that it can be send to the backend
        stateTransformer(state);
        axios.patch(TASK_LIST_ENDPOINT, taskListReorderEntries)
            .then(updateState)
            .catch(error => {
                console.error(`Failed to reorder task lists after dragging!`);
                console.error(error);
            });
        setState(stateTransformer);
    }

    function updateBoard(taskList: ITaskList, updatedTaskList: ITaskList) {
        if(taskList.id !== updatedTaskList.id)
        {
            console.error(`Can't patch task list ${taskList}: The id of the old and the id of the updated task list differ!`)
            return;
        }

        let patch = jsonpatch.compare(taskList, updatedTaskList);

        axios.patch(`${TASK_LIST_ENDPOINT}/${taskList.id}`, patch)
            .then(updateState)
            .catch(error => {
                console.error(`Failed to patch task list ${taskList}`);
                console.error(error);
            });

        let taskListIndex = state.taskLists.indexOf(taskList);
        setState(produce(draft => {
            draft.taskLists[taskListIndex] = updatedTaskList;
        }));
    }

    function taskListSorter(taskList1: ITaskList, taskList2: ITaskList) {
        return taskList1.index - taskList2.index;
    }

    function updateState() {
        return;
        axios.get(TASK_LIST_ENDPOINT)
            .then(response => {
                setState({
                    taskLists: response.data.sort(taskListSorter)
                })
            })
            .catch( error => {
                console.error('Failed to retrieve state!');
                console.error(error);
            });
    }

    function onDragEnd({destination, source, type}: DropResult) {
        if(!destination) return;

        if(destination.droppableId === source.droppableId
            && destination.index === source.index)
            return;

        if(type === TASK_LIST_DROPPABLE_TYPE) {
            handleTaskListMoved(source, destination);
            return;
        }

        let sourceTaskList = getTaskListViaId(source.droppableId);
        if(!sourceTaskList) {
            console.error(`Can't end drag: Failed to retrieve task list with droppableId: ${source.droppableId}!`);
            return;
        }

        let destinationTaskList = getTaskListViaId(destination.droppableId);
        if(!destinationTaskList) {
            console.error(`Can't end drag: Failed to retrieve task list with droppableId: ${source.droppableId}!`);
            return;
        }

        let task = sourceTaskList.tasks[source.index];
        if(!task) {
            console.error(`Can't end drag: Failed to retrieve task with index ${source.index} from taskList ${sourceTaskList}!`);
            return;
        }

        if(source.droppableId === destination.droppableId)
            handleTaskMovedInSameTaskList(sourceTaskList, source, destination);
        else
            handleTaskMovedToSeparateTaskList(sourceTaskList, destinationTaskList, task, source, destination);
    }
    
    return (
        <div className="container-fluid">
                <DragDropContext onDragEnd={onDragEnd}>
                    <Droppable droppableId='board' type={TASK_LIST_DROPPABLE_TYPE} direction='horizontal'>
                        { provided => (
                            <TaskListContainer
                                ref={provided.innerRef}
                                {...provided.droppableProps}
                            >
                                {state.taskLists
                                    .slice()
                                    .sort(taskListSorter)
                                    .map(taskList => (
                                        <TaskList
                                            key={taskList.id}
                                            taskList={taskList}
                                            onAddTaskToTaskList={handleAddTaskToTaskList}
                                            onRemoveTaskList={handleRemoveTaskList}
                                            onTaskListRenamed={handleTaskListRenamed}

                                            onRemoveTask={handleRemoveTask}
                                            onTaskIsDoneChanged={handleTaskIsDoneChanged}
                                            onTaskRenamed={handleTaskRenamed}
                                        />
                                    ))
                                }
                                {provided.placeholder}
                                <TaskListGenerator onGenerate={handleCreateNewTaskList}/>
                            </TaskListContainer>
                        )}
                    </Droppable>
                </DragDropContext>
            <p className="footer"> Created by Lukas Schwarz </p>
        </div>
    );
}
