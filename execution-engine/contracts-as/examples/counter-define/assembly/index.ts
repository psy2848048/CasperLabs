import {Error, ErrorCode} from "../../../../contract-as/assembly/error";
import {putKey, getKey, getArg, storeFunctionAtHash, ret} from "../../../../contract-as/assembly";
import {Key} from "../../../../contract-as/assembly/key";
import {CLValue} from "../../../../contract-as/assembly/clvalue";
import {toBytesString, toBytesMap, toBytesPair} from "../../../../contract-as/assembly/bytesrepr";
import {fromBytesString, fromBytesI32} from "../../../../contract-as/assembly/bytesrepr";
import {U512} from "../../../../contract-as/assembly/bignum";

const COUNT_KEY = "count";
const COUNTER_EXT = "counter_ext";
const COUNTER_KEY = "counter";
const GET_METHOD = "get";
const INC_METHOD = "inc";

enum Arg {
    UnknownMethodName = 0,
}

export function counter_ext(): void {
    const countKey = getKey(COUNT_KEY);
    if (countKey === null) {
        Error.fromErrorCode(ErrorCode.GetKey).revert();
        return;
    }

    const methodNameBytes = getArg(Arg.UnknownMethodName);
    if (methodNameBytes === null) {
        Error.fromErrorCode(ErrorCode.MissingArgument).revert();
        return;
    }

    const methodNameResult = fromBytesString(methodNameBytes);
    if (methodNameResult.hasError()) {
        Error.fromErrorCode(ErrorCode.InvalidArgument).revert();
        return;
    }
    const methodName = methodNameResult.value;

    if (methodName == INC_METHOD) {
        const oneValue = U512.fromU64(1);
        const one = CLValue.fromU512(oneValue);
        countKey.add(one);
    }
    else if (methodName == GET_METHOD) {
        const valueBytes = countKey.read();
        if (valueBytes === null) {
            Error.fromUserError(3).revert();
            return;
        }

        let i32Result = fromBytesI32(valueBytes);
        if (i32Result.hasError()) {
            Error.fromUserError(4).revert();
            return;
        }

        let returnValue = CLValue.fromI32(i32Result.value);
        ret(returnValue);
    }
    else {
        Error.fromUserError(0).revert();
    }
}

export function call(): void {
    let init = CLValue.fromI32(0);
    const maybeCounterLocalKey = Key.create(init);
    if (maybeCounterLocalKey === null) {
        Error.fromUserError(1).revert();
        return;
    }
    const counterLocalKey = <Key>maybeCounterLocalKey;

    let namedKeys = toBytesMap([
        toBytesPair(toBytesString(COUNT_KEY), counterLocalKey.toBytes()),
    ]);
    var pointer = storeFunctionAtHash(COUNTER_EXT, namedKeys);
    putKey(COUNTER_KEY, pointer);
}
