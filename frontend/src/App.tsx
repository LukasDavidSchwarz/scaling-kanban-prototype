import React, { useEffect } from 'react';
import './App.css';
import TaskListGenerator from './TaskListGenerator';
import produce from 'immer';
import TaskList from './TaskList';
import axios from 'axios';
import useStateCallback from './useStateCallback';
import { DragDropContext, DraggableLocation, Droppable, DropResult } from 'react-beautiful-dnd';
import styled from 'styled-components';
import { v4 as uuidv4 } from 'uuid';
import { IBoard, ITask, ITaskList } from './types';
import useWebSocket from 'react-use-websocket';

interface State {
    boardId?: string;
    board: IBoard;
}

//TODO: use environment variable to determine api endpoint
const TASK_LIST_ENDPOINT = 'http://0.0.0.0:3512/board';

const TASK_LIST_DROPPABLE_TYPE = 'TASK_LIST';

const TaskListContainer = styled.div`
    display: flex;
`;

export default function App() {
    const [state, setState] = useStateCallback<State>({
        boardId: undefined,
        board: {
            url: '-',
            version: -1,
            name: '-',
            lists: [],
        },
    });

    const { lastJsonMessage } = useWebSocket(`ws://localhost:3512/board/${state.boardId}/watch`, {
        onOpen: (_) => console.log('Websocket opened'),
        onMessage: (message) => console.log(`Received ${message.data}`),
        shouldReconnect: (_) => true,
    });

    // eslint-disable-next-line react-hooks/exhaustive-deps
    useEffect(fetchBoard, []); // empty dependency array so that hook is only called when component is mounted/unmounted

    const urlParams = new URLSearchParams(window.location.search);
    const boardId = urlParams.get('boardId');
    if (boardId === null) {
        console.error('Failed to retrieve board_id query parameter!');
    } else if (boardId !== state.boardId) {
        setState(
            produce((draft) => {
                draft.boardId = boardId;
            })
        );
    }

    handleSocketMessage(lastJsonMessage);

    if (state.boardId === null) {
        return (
            <div className="container-fluid">
                <p>No board selected!</p>
            </div>
        );
    }

    function updateTaskList(
        taskList: ITaskList,
        taskListProducer: (state?: ITaskList | undefined) => ITaskList,
        preview: boolean
    ) {
        let taskListIndex = state.board.lists.indexOf(taskList);
        const boardProducer = produce<IBoard>((draft) => {
            draft.lists[taskListIndex] = taskListProducer(taskList);
        });

        updateBoard(boardProducer, preview);
    }

    function updateBoard(boardProducer: (state?: IBoard | undefined) => IBoard, preview: boolean) {
        if (!state.board) return;

        const updatedBoard = boardProducer(state.board);
        axios
            .put(`${TASK_LIST_ENDPOINT}/${state.board.id}`, updatedBoard)
            .then((response) => {
                const board = processBoardFromExternalSource(response.data);
                setState(
                    produce((draft) => {
                        draft.board = board;
                    })
                );
            })
            .catch((error) => {
                console.error(`Failed to patch task list ${updatedBoard}`);
                console.error(error);
            });

        if (preview) {
            setState(
                produce((draft) => {
                    draft.board = boardProducer(state.board);
                })
            );
        }
    }

    function fetchBoard() {
        if (state.boardId === null) return;

        axios
            .get(`${TASK_LIST_ENDPOINT}/${state.boardId}`)
            .then((response) => {
                const board = processBoardFromExternalSource(response.data);
                setState(
                    produce((draft) => {
                        draft.board = board;
                    })
                );
            })
            .catch((error) => {
                console.error('Failed to retrieve state!');
                console.error(error);
            });
    }

    function handleSocketMessage(jsonMessage: any) {
        if (!lastJsonMessage) return;
        const newBoard = processBoardFromExternalSource(jsonMessage as IBoard);

        if (newBoard.id !== state.board.id) {
            console.warn('Received board with wrong id from socket:', newBoard);
            return;
        }
        if (newBoard.version <= state.board.version) {
            console.debug(
                `Received board version '${newBoard.version}' witch is lower or equal to current version '${state.board.version}':`,
                newBoard
            );
            return;
        }

        console.debug('Updating board with version sent by socket:', newBoard);
        setState(
            produce((draft) => {
                draft.board = newBoard;
            })
        );
    }

    function processBoardFromExternalSource(board: IBoard) {
        // TODO: Validate json
        produce(board, (draft) => {
            draft.lists.forEach((list) =>
                list.tasks.forEach((task) => {
                    // @ts-ignore
                    task.isDone = task['is_done'];
                    // @ts-ignore
                    task['is_done'] = undefined;
                })
            );
        });
        return board;
    }

    function handleCreateNewTaskList(name = 'New ToDo List') {
        let newTaskList: ITaskList = {
            id: uuidv4(),
            name: name,
            tasks: [],
        };

        const boardProducer = produce<IBoard>((draft) => {
            draft.lists.push(newTaskList);
        });
        updateBoard(boardProducer, false);
    }

    function handleAddTaskToTaskList(taskList: ITaskList, name = 'New Task'): void {
        let newTask: ITask = {
            id: uuidv4(),
            name,
        };

        const taskListProducer = produce<ITaskList>((draft) => {
            draft.tasks.push(newTask);
        });
        updateTaskList(taskList, taskListProducer, false);
    }

    function handleRemoveTaskList(taskList: ITaskList): void {
        let taskListIndex = state.board.lists.indexOf(taskList);
        let boardProducer = produce<IBoard>((draft) => {
            draft.lists.splice(taskListIndex, 1);
        });
        updateBoard(boardProducer, false);
    }

    function handleRemoveTask(taskList: ITaskList, task: ITask): void {
        let taskIndex = taskList.tasks.indexOf(task);
        let taskListProducer = produce<ITaskList>((draft) => {
            draft.tasks.splice(taskIndex, 1);
        });
        updateTaskList(taskList, taskListProducer, false);
    }

    function handleTaskListRenamed(taskList: ITaskList, newName: string): void {
        let taskListProducer = produce<ITaskList>((draft) => {
            draft.name = newName;
        });
        updateTaskList(taskList, taskListProducer, true);
    }

    function handleTaskRenamed(taskList: ITaskList, task: ITask, newName: string): void {
        let taskIndex = taskList.tasks.indexOf(task);

        let taskListProducer = produce<ITaskList>((draft) => {
            draft.tasks[taskIndex].name = newName;
        });

        updateTaskList(taskList, taskListProducer, true);
    }

    function handleTaskMovedInSameTaskList(
        taskList: ITaskList,
        source: DraggableLocation,
        destination: DraggableLocation
    ) {
        let taskListProducer = produce<ITaskList>((draft) => {
            let [removedTask] = draft.tasks.splice(source.index, 1);
            draft.tasks.splice(destination.index, 0, removedTask);
        });

        updateTaskList(taskList, taskListProducer, true);
    }

    function handleTaskMovedToSeparateTaskList(
        sourceTaskList: ITaskList,
        destinationTaskList: ITaskList,
        task: ITask,
        source: DraggableLocation,
        destination: DraggableLocation
    ) {
        const sourceListIndex = state.board.lists.indexOf(sourceTaskList);
        const destinationListIndex = state.board.lists.indexOf(destinationTaskList);
        const boardProducer = produce<IBoard>((draft) => {
            draft.lists[sourceListIndex].tasks.splice(source.index, 1);
            draft.lists[destinationListIndex].tasks.splice(destination.index, 0, task);
        });

        updateBoard(boardProducer, true);
    }

    function handleTaskListMoved(source: DraggableLocation, destination: DraggableLocation) {
        let boardProducer = produce<IBoard>((draft) => {
            let [movedList] = draft.lists.splice(source.index, 1);
            draft.lists.splice(destination.index, 0, movedList);
        });

        updateBoard(boardProducer, true);
    }

    function onDragEnd({ destination, source, type }: DropResult) {
        if (!destination) return;

        if (destination.droppableId === source.droppableId && destination.index === source.index)
            return;

        if (type === TASK_LIST_DROPPABLE_TYPE) {
            handleTaskListMoved(source, destination);
            return;
        }

        let sourceTaskList = getTaskListViaId(source.droppableId);
        if (!sourceTaskList) {
            console.error(
                `Can't end drag: Failed to retrieve task list with droppableId: ${source.droppableId}!`
            );
            return;
        }

        let destinationTaskList = getTaskListViaId(destination.droppableId);
        if (!destinationTaskList) {
            console.error(
                `Can't end drag: Failed to retrieve task list with droppableId: ${source.droppableId}!`
            );
            return;
        }

        let task = sourceTaskList.tasks[source.index];
        if (!task) {
            console.error(
                `Can't end drag: Failed to retrieve task with index ${source.index} from taskList ${sourceTaskList}!`
            );
            return;
        }

        if (source.droppableId === destination.droppableId)
            handleTaskMovedInSameTaskList(sourceTaskList, source, destination);
        else
            handleTaskMovedToSeparateTaskList(
                sourceTaskList,
                destinationTaskList,
                task,
                source,
                destination
            );

        function getTaskListViaId(taskListId: string) {
            return state.board?.lists.find((taskList) => taskList.id?.toString() === taskListId);
        }
    }

    return (
        <div className="container-fluid">
            <DragDropContext onDragEnd={onDragEnd}>
                <Droppable
                    droppableId="board"
                    type={TASK_LIST_DROPPABLE_TYPE}
                    direction="horizontal"
                >
                    {(provided) => (
                        <TaskListContainer ref={provided.innerRef} {...provided.droppableProps}>
                            {state.board?.lists &&
                                state.board.lists
                                    .slice()
                                    .map((taskList, index) => (
                                        <TaskList
                                            key={taskList.id}
                                            taskList={taskList}
                                            index={index}
                                            onAddTaskToTaskList={handleAddTaskToTaskList}
                                            onRemoveTaskList={handleRemoveTaskList}
                                            onTaskListRenamed={handleTaskListRenamed}
                                            onRemoveTask={handleRemoveTask}
                                            onTaskRenamed={handleTaskRenamed}
                                        />
                                    ))}
                            {provided.placeholder}
                            <TaskListGenerator onGenerate={handleCreateNewTaskList} />
                        </TaskListContainer>
                    )}
                </Droppable>
            </DragDropContext>
            <p className="footer"> Created by Lukas Schwarz </p>
        </div>
    );
}
