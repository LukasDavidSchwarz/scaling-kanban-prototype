import './Task.css';
import RenameInput from './RenameInput';
import { Draggable } from 'react-beautiful-dnd';
import styled from 'styled-components';
import { ITask } from './types';

interface Props {
    task: ITask;
    index: number;
    onRenamed: (task: ITask, newName: string) => void;
    onRemoveTask: (task: ITask) => void;
}

const Container = styled.div`
    cursor: pointer;
`;

export default function Task({ task, index, onRemoveTask, onRenamed }: Props) {
    return (
        <Draggable draggableId={task?.id ?? ''} index={index}>
            {(provided) => (
                <Container
                    {...provided.draggableProps}
                    {...provided.dragHandleProps}
                    ref={provided.innerRef}
                >
                    <div className="task gray-light row mx-0 my-1 px-0 rounded">
                        <RenameInput
                            actualName={task.name}
                            onRenamed={(newName) => onRenamed(task, newName)}
                            className="taskNameInput"
                        />
                        <button
                            onClick={() => onRemoveTask(task)}
                            type="button"
                            className="removeTaskButton btn btn-gray-light col-auto m-1"
                        >
                            <i className="fa fa-trash" />
                        </button>
                    </div>
                </Container>
            )}
        </Draggable>
    );
}
