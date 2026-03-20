/**
 * JSON Pointer (RFC 6901) helpers using json-ptr library.
 */
import { JsonPointer } from 'json-ptr';

/**
 * Resolve a JSON Pointer path to a value in an object.
 * Empty pointer "" returns the root object.
 */
export function resolvePointer(obj: Record<string, unknown>, pointer: string): unknown {
	if (pointer === '') return obj;
	return JsonPointer.get(obj, pointer);
}

/**
 * Set a value at a JSON Pointer path.
 * If value is null, deletes the key from the parent object.
 * If create is true (default), creates intermediate objects as needed.
 */
export function setAtPointer(
	obj: Record<string, unknown>,
	pointer: string,
	value: unknown,
	create: boolean = true
): void {
	if (pointer === '') return; // Cannot replace root via pointer

	if (value === null) {
		// Delete the key from the parent object
		const segments = JsonPointer.decode(pointer);
		const key = segments.pop();
		if (key === undefined) return;

		const parentPointer = segments.length === 0 ? '' : JsonPointer.create(segments).pointer;
		const parent = parentPointer === '' ? obj : (JsonPointer.get(obj, parentPointer) as Record<string, unknown>);
		if (parent && typeof parent === 'object') {
			delete (parent as Record<string, unknown>)[key];
		}
		return;
	}

	JsonPointer.set(obj, pointer, value, create);
}
