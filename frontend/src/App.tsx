import React, { Fragment } from 'react';
import './App.css';
import { Route, Routes } from 'react-router-dom';
import Board from './Board';
import BoardList from './BoardList';

export default function App() {
    return (
        <Fragment>
            <Routes>
                <Route path="/" element={<BoardList />} />
                <Route path="/boards/:boardId" element={<Board />} />
            </Routes>
            <p className="footer"> Created by Lukas Schwarz </p>
        </Fragment>
    );
}
