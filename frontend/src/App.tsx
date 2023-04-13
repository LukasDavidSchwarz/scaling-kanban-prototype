import React, { Fragment } from 'react';
import './App.css';
import { Route, Routes } from 'react-router-dom';
import Board from './Board';
import BoardList from './BoardList';
import Header from './Header';
import Footer from './Footer';

export default function App() {
    return (
        <Fragment>
            <Header />
            <Routes>
                <Route path="/" element={<BoardList />} />
                <Route path="/boards/:boardId" element={<Board />} />
            </Routes>
            <Footer />
        </Fragment>
    );
}
