// created by: https://www.joshwcomeau.com/
// source: https://www.joshwcomeau.com/react/persisting-react-state-in-localstorage/

import React, {useState} from "react";

export default function useStickyState<T>(defaultValue: T, key: string): [T,  React.Dispatch<React.SetStateAction<T>>] {
    const [value, setValue] = useState(() => {
        const stickyValue = window.localStorage.getItem(key);
        return stickyValue !== null
            ? JSON.parse(stickyValue) as T
            : defaultValue;
    });
    React.useEffect(() => {
        window.localStorage.setItem(key, JSON.stringify(value));
    }, [key, value]);
    return [value, setValue];
}