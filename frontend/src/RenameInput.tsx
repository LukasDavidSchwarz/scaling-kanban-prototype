import React, { ChangeEvent, useRef } from 'react';
import useStateCallback from './useStateCallback';
import styled from 'styled-components';
import TextAreaAutosize from 'react-textarea-autosize';

interface Props {
    actualName: string;
    onRenamed: (newName: string) => void;
    className: string;
}

const FocusOnClickProxy = styled.div`
    position: absolute;
    width: 100%;
    height: 100%;
    top: 0;
    left: 0;
    cursor: pointer;
`;

/**
 * Stores temporal changes made in it's input field and applies or reverts them when the input field looses focus.
 */
export default function RenameInput({ className, actualName, onRenamed }: Props) {
    const [tempName, setTempName] = useStateCallback<string | null>(null);
    const textAreaRef = useRef<HTMLTextAreaElement>(null);

    function handleBlur() {
        if (tempName === null) return;

        let newName = tempName.trim();
        if (newName === '') newName = actualName;

        if (newName !== actualName) onRenamed(newName);

        setTempName(null);
    }

    function handleChange(event: ChangeEvent<HTMLTextAreaElement>) {
        setTempName(event.target.value);
    }

    function handleKeyDown(event: React.KeyboardEvent) {
        if (textAreaRef.current == null) return;

        if (event.key === 'Enter') textAreaRef.current.blur();
        else if (event.key === 'Escape') {
            setTempName(null, () => {
                if (textAreaRef.current != null) textAreaRef.current.blur();
            });
        }
    }

    function handleFocus() {
        if (textAreaRef.current === null) return;
        textAreaRef.current.select();
        setTempName(actualName);
    }

    function onFocusOnClickProxyClicked() {
        if (textAreaRef.current === null) return;
        textAreaRef.current.focus();
    }

    return (
        <div
            className="col my-1 mx-1 px-1 py-0"
            style={{ position: 'relative', cursor: 'pointer' }}
        >
            <TextAreaAutosize
                ref={textAreaRef}
                value={tempName === null ? actualName : tempName}
                placeholder={actualName}
                onFocus={handleFocus}
                onChange={handleChange}
                onBlur={handleBlur}
                onKeyDown={handleKeyDown}
                className={`nameInput ${className} my-1`}
            />
            {!tempName && <FocusOnClickProxy onClick={onFocusOnClickProxyClicked} />}
        </div>
    );
}
