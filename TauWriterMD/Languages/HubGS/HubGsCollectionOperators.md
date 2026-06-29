# HubGS Collection Operators

To support scalable scripting inside `@computed` property functions, Array arrays and structural roles expose functional chain modifiers within the active evaluation layer.

## Chain Methods

### `.len()`
Returns the total integer length of items within an array context or relationship role mapping.
* **Returns**: `Number`
* **Usage**: `companion_count = @computed(this.companions.len())`

### `.map(expression)`
Transforms every member inside a targeted array block using a mapped property assignment.
* **Returns**: `Array<T>`
* **Usage**: `this.companions.map(c => c.first_name)`

### `.join(delimiter)`
Flattens an array collection of text into a single cohesive string, splitting elements cleanly via an explicit separator character.
* **Returns**: `Text`
* **Usage**: `party_manifest = @computed(this.companions.map(c => c.first_name).join(", "))`
