import React from "react";

interface Props {
    onGenerate: () => void;
}

export default function TaskListGenerator({onGenerate}: Props) {
    return (
        <button
            onClick={() => onGenerate()}
            type="button"
            className="btn taskList addTaskListButton btn-block text-left my-2 mx-1"
        >
            + Add new list
        </button>
    );
};