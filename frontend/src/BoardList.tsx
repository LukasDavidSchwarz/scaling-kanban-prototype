import { Link } from 'react-router-dom';
import React, { useState } from 'react';
import axios from 'axios';
import { IBoard } from './types';

interface State {
    boards: IBoard[];
}

const env = import.meta.env;
const REST_API_URL = `${env.VITE_REST_API_PROTOCOL}${env.VITE_API_HOST}/api/v1`;

export default function BoardList() {
    const [state, setState] = useState<State>({
        boards: [],
    });

    // TODO: Move this into periodic callback
    axios
        .get(`${REST_API_URL}/boards`)
        .then((resp) => {
            // TODO: Validate json
            let boards = resp.data as IBoard[];
            setState((_) => ({
                boards,
            }));
        })
        .catch((err) => {
            console.error('Failed to retrieve boards!', err);
        });

    return (
        <div className="container">
            <h2 className="mt-5 text-center">Boards:</h2>
            <div className="card-columns">
                {state.boards.map((board) => (
                    <Link
                        key={board.id}
                        className="card board-link background-blue"
                        to={`/boards/${board.id}`}
                    >
                        <div className="card-body">{board.name}</div>
                    </Link>
                ))}
            </div>
        </div>
    );
}
