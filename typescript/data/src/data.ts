
import { SHA3 } from "sha3";

/**
 * {@link Datum} represents an arbitrary document. Instances are generated
 * pseudo-randomly by the `httpd-a` service and placed into Kinesis for
 * consumption by the `events-a` service. `events-a` compute an iterative
 * SHA-512 hash of the document prior to re-injecting it into Kinesis. Finally,
 * `events-b` consumes the document and stores it in DynamoDB.
 */
export class Datum
{
	/**
	 * The unique identifier for this document, used as a partition key in
	 * DynamoDB.
	 */
	#uuid: string;

	/**
	 * The pseudo-randomly generated document, produced by `httpd-a`. The
	 * generator is only defined in the Rust project.
	 */
	#doc: string;

	/**
	 * The target iteration count for the SHA-512 hash.
	 */
	#hashes: number;

	/**
	 * The hash of the document, computed by `events-a`.
	 */
	#hash?: string | null;

	/**
	 * Construct a new `Datum`.
	 *
	 * @param uuid
	 *   The unique identifier for this document, used as a partition key in
	 *   DynamoDB.
	 * @param doc
	 *   The pseudo-randomly generated document, produced by `httpd-a`.
	 * @param hashes
	 *   The target iteration count for the SHA-512 hash.
	 * @param hash
	 *   The hash of the document, computed by `events-a`.
	 */
	constructor(uuid: string, doc: string, hashes: number, hash?: string)
	{
		this.#uuid = uuid;
		this.#doc = doc;
		this.#hashes = hashes;
		this.#hash = hash;
	}

	/**
	 * Answer the unique identifier for the receiver.
	 *
	 * @returns
	 *   The unique identifier.
	 */
	public uuid = (): string => this.#uuid;

	/**
	 * Answer the document.
	 *
	 * @returns
	 *   The document.
	 */
	public doc = (): string => this.#doc;

	/**
	 * Answer the target iteration count for the SHA-512 hash.
	 *
	 * @returns
	 *   The target iteration count for the SHA-512 hash.
	 */
	public hashes = (): number => this.#hashes;

	/**
	 * Lazily compute the iterative SHA-512 hash of the document.
	 *
	 * @returns
	 *   The hash of the document.
	 */
	public hash = (): string =>
	{
		if (this.#hash === undefined || this.#hash === null)
		{
			const hash = new SHA3(512);
			for (let i = 0; i < this.#hashes; i++)
			{
				hash.update(this.#doc);
			}
			this.#hash = hash.digest("hex").toUpperCase();
		}
		return this.#hash;
	};

	/**
	 * Construct a new `Datum` from a JSON string.
	 *
	 * @param json
	 *   The JSON string.
	 * @returns
	 *   The deserialized `Datum`.
	 */
	public static fromJSON = (json: string): Datum | undefined =>
	{
		const obj: any = JSON.parse(json);
		console.trace("Parsed JSON: ", obj);
		if (obj.uuid === undefined
			|| obj.doc === undefined
			|| obj.hashes === undefined
			|| typeof obj.hashes !== "number")
		{
			return undefined;
		}
		return new Datum(obj.uuid, obj.doc, obj.hashes, obj.hash);
	};

	/**
	 * Answer the JSON surrogate of the receiver.
	 *
	 * @returns
	 *   The JSON surrogate of the receiver.
	 */
	public toJSON = (): object => ({
		uuid: this.#uuid,
		doc: this.#doc,
		hashes: this.#hashes,
		hash: this.#hash
	});

	/**
	 * Answer the UUID as the debug representation of the receiver.
	 *
	 * @returns
	 *   The UUID.
	 */
	public toString = (): string =>
	{
		if (this.#hash === undefined || this.#hash === null)
		{
			return `doc: ${this.#doc}`;
		}
		else
		{
			return `hash: ${this.#hash}`;
		}
	};
}
