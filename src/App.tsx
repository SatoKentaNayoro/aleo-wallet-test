import React, {useEffect, useState} from 'react';
import init, {request_records} from "wasm-lib";
import './App.css';

const App = () => {
    const [privateKey, setPrivateKey] = useState<string | undefined>(undefined);
    const [viewKey, setViewKey] = useState('');
    const [start, setStart] = useState<number | undefined>(undefined);
    const [end, setEnd] = useState<number | undefined>(undefined);
    const [last, setLast] = useState<number | undefined>(undefined);
    const [endpoint, setEndpoint] = useState('');

    useEffect(() => {
        init();
    }, []);

    const handleSubmit = async (event: React.FormEvent<HTMLFormElement>) => {
        event.preventDefault();
        console.log({
            private_key: privateKey || undefined,
            view_key: viewKey,
            start: start || undefined,
            end: end || undefined,
            last: last || undefined,
            endpoint: endpoint,
        });
        try {
            const records = request_records(privateKey, viewKey, start, end, last, endpoint);
            console.log(records);
        } catch (error) {
            console.error("Failed to request records:", error);
        }

    };

    const handleInputChange = <T, >(
        event: React.ChangeEvent<HTMLInputElement>,
        setState: React.Dispatch<React.SetStateAction<T>>
    ) => {
        const value = event.target.value;
        if (event.target.type === 'number') {
            setState((value === '' ? undefined : Number(value)) as T);
        } else {
            setState(value as T);
        }
    };


    return (
        <div>
            <h1>请输入参数</h1>
            <form onSubmit={handleSubmit}>
                <label htmlFor="private_key">Private Key (可选):</label>
                <input
                    type="text"
                    id="private_key"
                    name="private_key"
                    placeholder="请输入PrivateKey"
                    value={privateKey === undefined ? '' : privateKey}
                    onChange={(e) => handleInputChange<string | undefined>(e, setPrivateKey)}
                />
                <br/>
                <br/>

                <label htmlFor="view_key">View Key:</label>
                <input
                    type="text"
                    id="view_key"
                    name="view_key"
                    placeholder="请输入查ViewKey"
                    required
                    value={viewKey}
                    onChange={(e) => handleInputChange<string>(e, setViewKey)}
                />
                <br/>
                <br/>

                <label htmlFor="start">Start (可选):</label>
                <input
                    type="number"
                    id="start"
                    name="start"
                    placeholder="请输入开始值"
                    value={start === undefined ? '' : start}
                    onChange={(e) => handleInputChange<number | undefined>(e, setStart)}
                />
                <br/>
                <br/>

                <label htmlFor="end">End (可选):</label>
                <input
                    type="number"
                    id="end"
                    name="end"
                    placeholder="请输入结束值"
                    value={end}
                    onChange={(e) => handleInputChange<number | undefined>(e, setEnd)}
                />
                <br/>
                <br/>

                <label htmlFor="last">Last (可选):</label>
                <input
                    type="number"
                    id="last"
                    name="last"
                    placeholder="请输入最后值"
                    value={last}
                    onChange={(e) => handleInputChange<number | undefined>(e, setLast)}
                />
                <br/>
                <br/>

                <label htmlFor="endpoint">Endpoint:</label>
                <input
                    type="text"
                    id="endpoint"
                    name="endpoint"
                    placeholder="请输入Endpoint"
                    required
                    value={endpoint}
                    onChange={(e) => handleInputChange<string>(e, setEndpoint)}
                />
                <br/>
                <br/>

                <input type="submit" value="Submit"/>
            </form>
        </div>
    );
};

export default App;
