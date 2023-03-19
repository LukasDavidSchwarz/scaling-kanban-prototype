// Created by https://stackoverflow.com/users/10199138/mj-studio
// Source: https://stackoverflow.com/a/61725731/12113115

import { SetStateAction, useCallback, useEffect, useRef, useState } from 'react';

type Callback<T> = (value?: T) => void;
type DispatchWithCallback<T> = (value: T, callback?: Callback<T>) => void;

function useStateCallback<T>(initialState: T | (() => T)): [T, DispatchWithCallback<SetStateAction<T>>] {
    const [state, _setState] = useState(initialState);

    const callbackRef = useRef<Callback<T>>();
    const isFirstCallbackCall = useRef<boolean>(true);

    const setState = useCallback((setStateAction: SetStateAction<T>, callback?: Callback<T>): void => {
        callbackRef.current = callback;
        _setState(setStateAction);
    }, []);

    useEffect(() => {
        if (isFirstCallbackCall.current) {
            isFirstCallbackCall.current = false;
            return;
        }
        callbackRef.current?.(state);
    }, [state]);

    return [state, setState];
}

export default useStateCallback;