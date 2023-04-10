import { Link } from 'react-router-dom';
import React, { Fragment } from 'react';
import useStateCallback from './useStateCallback';
import axios from 'axios';
import { IBoard } from './types';

interface State {
    boards?: IBoard[];
}

const env = import.meta.env;
const REST_API_URL = `${env.VITE_REST_API_PROTOCOL}${env.VITE_API_HOST}/api/v1`;

export default function BoardList() {
    const [state, setState] = useStateCallback<State>({
        boards: undefined,
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

    if (!state.boards) {
        return (
            <Fragment>
                <h2> Loading boards...</h2>
            </Fragment>
        );
    }

    return (
        <Fragment>
            <h2>Boards:</h2>
            {state.boards.map((board, index) => (
                <Link
                    className="dropdown-item"
                    to={`/boards/${board.id}`}
                    style={{ color: 'white' }}
                >
                    {board.name}
                </Link>
            ))}
        </Fragment>
    );
}
