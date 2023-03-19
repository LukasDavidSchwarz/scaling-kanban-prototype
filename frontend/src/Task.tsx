import RenameInput from "./RenameInput";
import {Draggable} from "react-beautiful-dnd";
import styled from "styled-components";
import {ITask} from "./types";

interface Props {
    task: ITask;
    index: number;
    onIsDoneChanged: (task: ITask, newVal: boolean) => void;
    onRenamed: (task: ITask, newName: string) => void;
    onRemoveTask: (task: ITask) => void;
}

const STRIKETHROUGH_CLASS = 'strikethrough';

const Container = styled.div`
  cursor: pointer;
`;

export default function Task({task, index, onIsDoneChanged, onRemoveTask, onRenamed}: Props) {
    return (
        <Draggable draggableId={task.id.toString()} index={index}>
            {provided => (
                <Container
                    {...provided.draggableProps}
                    {...provided.dragHandleProps}
                    ref={provided.innerRef}
                >
                    <div className="task gray-light row mx-0 my-1 pl-3 pr-0 rounded">
                        <input
                            checked={task.isDone}
                            onChange={(e) => onIsDoneChanged(task, e.target.checked)}
                            type="checkbox"
                            className="taskCheckbox my-auto big-checkbox col-auto my-1"
                            id="flexCheckDefault"
                        />
                        <RenameInput
                            actualName={task.name}
                            onRenamed={(newName) => onRenamed(task, newName)}
                            className={`taskNameInput ${task.isDone ? STRIKETHROUGH_CLASS : ''}`}
                        />
                        <button onClick={() => onRemoveTask(task)}
                                type="button"
                                className="removeTaskButton btn btn-gray-light col-auto m-1"
                        >
                            <i className='fa fa-trash'/>
                        </button>
                    </div>
                </Container>
            )}
        </Draggable>
    );
}