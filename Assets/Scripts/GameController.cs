namespace Assets.Scripts.Core
{
    using System;
    using System.Collections.Generic;
    using System.Linq;
    using Assets.Scripts.ScriptableObject;
    using Assets.Scripts.WorldObjects.Core;

    public class GameControllerCore
    {
        public Dictionary<
            System.Numerics.Vector2,
            Dictionary<string, WorldObjectCore>
        > worldObjects = new();
        public List<DeletionQueueItem> queuedForDeletion = new();
        public List<SpawnQueueItem> queuedForSpawn = new();
        public List<MovementQueueItem> queuedForMovement = new();

        // CLASSES //

        public class MisconfigurationException : Exception
        {
            public MisconfigurationException(string message)
                : base(message) { }
        }

        public class SpawnException : Exception
        {
            public SpawnException(string message)
                : base(message) { }
        }

        public class DeletionQueueItem
        {
            public WorldObjectCore worldObject;
            public System.Numerics.Vector2 position;

            public DeletionQueueItem(WorldObjectCore worldObject, System.Numerics.Vector2 position)
            {
                this.worldObject = worldObject;
                this.position = position;
            }
        }

        public class MovementQueueItem
        {
            public System.Numerics.Vector2 oldPosition;
            public System.Numerics.Vector2 newPosition;
            public WorldObjectCore worldObject;

            public MovementQueueItem(
                System.Numerics.Vector2 oldPosition,
                System.Numerics.Vector2 newPosition,
                WorldObjectCore worldObject
            )
            {
                this.oldPosition = oldPosition;
                this.newPosition = newPosition;
                this.worldObject = worldObject;
            }
        }

        // FUNCTIONS //

        public List<WorldObjectCore> GetAdjacentWorldObjects(System.Numerics.Vector2 position)
        {
            return new List<System.Numerics.Vector2>
                {
                    new( // Center
                        position.X,
                        position.Y
                    ),
                    new( // Above
                        position.X + 0,
                        position.Y + 1
                    ),
                    new( // Top Right
                        position.X + 1,
                        position.Y + 1
                    ),
                    new( // Right
                        position.X + 1,
                        position.Y + 0
                    ),
                    new( // Bottom Right
                        position.X + 1,
                        position.Y - 1
                    ),
                    new( // Below
                        position.X + 0,
                        position.Y - 1
                    ),
                    new( // Bottom Left
                        position.X - 1,
                        position.Y - 1
                    ),
                    new( // Left
                        position.X + -1,
                        position.Y + 0
                    ),
                    new( // Top Left
                        position.X + -1,
                        position.Y + 1
                    ),
                }
                    .Select(adjacentTile => this.GetWorldObjectsByPosition(adjacentTile))
                    .Where(worldObjects => worldObjects != null)
                    .SelectMany(worldObjects => worldObjects)
                    .Where(worldObject => worldObject != null)
                    .Distinct()
                    .ToList() ?? new List<WorldObjectCore>();
        }

        public List<WorldObjectCore> GetWorldObjectsByPosition(System.Numerics.Vector2 position)
        {
            Dictionary<string, WorldObjectCore> worldObjects = this.worldObjects.GetValueOrDefault(
                position,
                null
            );

            return worldObjects?.Values.ToList();
        }

        public List<WorldObjectCore> GetWorldObjectsByPositionAndType(
            System.Numerics.Vector2 position,
            List<string> types
        )
        {
            List<WorldObjectCore> worldObjects = this.GetWorldObjectsByPosition(position);

            // Nothing is here
            if (worldObjects == null)
            {
                return null;
            }

            // Find the things that match the type
            List<WorldObjectCore> matchingWorldObjects = new();
            foreach (WorldObjectCore worldObject in worldObjects)
            {
                if (types.Contains(worldObject.worldObjectType))
                {
                    matchingWorldObjects.Add(worldObject);
                }
            }
            return matchingWorldObjects;
        }
    }
}

#if UNITY_6000
namespace Assets.Scripts.Unity
{
    using System.Collections.Generic;
    using Assets.Scripts.Components.Unity;
    using Assets.Scripts.Core;
    using Assets.Scripts.ScriptableObject;
    using Assets.Scripts.WorldObjects.Core;
    using Assets.Scripts.WorldObjects.Unity;
    using Sirenix.OdinInspector;
    using UnityEngine;

    public class GameController : SerializedMonoBehaviour
    {
        public GameControllerCore core;

        public GameObject spawnables;
        public GameObject userInterface;
        public float tickFrequency = 0.1f;
        public int randomSeed = 0;
        public System.Random random;
        public bool readyForTicks = false;
        public SpriteMapComponent Map { get; set; }
        public PlayerComponent PlayerComponent { get; set; }
        public float lastTick = 0;

        // PROPERTIES //

        public int Tick { get; protected set; } = 0;
        public Dictionary<
            System.Numerics.Vector2,
            Dictionary<string, WorldObjectCore>
        > WorldObjects => this.core.worldObjects;

        // FUNCTIONS //

        public virtual void Start()
        {
            this.core = new GameControllerCore();
        }

        public virtual void Update()
        {
            // This is the main game loop.
            // If we aren't ready for ticks, the main game loop won't run.
            if (this.readyForTicks && (Time.time > this.lastTick + this.tickFrequency))
            {
                // Tick all objects
                foreach (
                    Dictionary<string, WorldObjectCore> worldObjects in this.core
                        .worldObjects
                        .Values
                )
                {
                    foreach (WorldObjectCore worldObject in worldObjects.Values)
                    {
                        (worldObject.backref as WorldObject).Tick(this);
                    }
                }

                // Delete queued objects
                if (this.core.queuedForDeletion != null)
                {
                    foreach (
                        GameControllerCore.DeletionQueueItem deletionQueueItem in this.core.queuedForDeletion
                    )
                    {
                        this.Delete(deletionQueueItem);
                    }
                    this.core.queuedForDeletion.Clear();
                }

                // TODO: make sure an object isnt being moved while being deleted.
                // TODO: LERP the movement of objects
                // Move queued objects
                if (this.core.queuedForMovement != null)
                {
                    foreach (
                        GameControllerCore.MovementQueueItem movementQueueItem in this.core.queuedForMovement
                    )
                    {
                        this.Move(movementQueueItem);
                    }
                    this.core.queuedForMovement.Clear();
                }

                // Spawn queued objects
                if (this.core.queuedForSpawn != null)
                {
                    foreach (SpawnQueueItem spawnQueueItem in this.core.queuedForSpawn)
                    {
                        this.Spawn(spawnQueueItem);
                    }
                    this.core.queuedForSpawn.Clear();
                }

                // Handle ticks
                this.lastTick = Time.time;
                this.Tick++;
            }
        }

        public List<WorldObjectCore> GetAdjacentWorldObjects(System.Numerics.Vector2 position) =>
            this.core.GetAdjacentWorldObjects(position);

        public List<WorldObjectCore> GetWorldObjectsByPosition(System.Numerics.Vector2 position) =>
            this.core.GetWorldObjectsByPosition(position);

        public List<WorldObjectCore> GetWorldObjectsByPositionAndType(
            System.Numerics.Vector2 position,
            List<string> types
        ) => this.core.GetWorldObjectsByPositionAndType(position, types);

        public void QueueForMovement(GameControllerCore.MovementQueueItem movementQueueItem)
        {
            this.core.queuedForMovement ??= new List<GameControllerCore.MovementQueueItem>();
            this.core.queuedForMovement.Add(movementQueueItem);
        }

        public void QueueForDeletion(GameControllerCore.DeletionQueueItem deletionQueueItem)
        {
            this.core.queuedForDeletion ??= new List<GameControllerCore.DeletionQueueItem>();
            this.core.queuedForDeletion.Add(deletionQueueItem);
        }

        public void QueueForSpawn(SpawnQueueItem spawnQueueItem)
        {
            this.core.queuedForSpawn ??= new List<SpawnQueueItem>();
            this.core.queuedForSpawn.Add(spawnQueueItem);
        }

        protected virtual void Reset()
        {
            // TODO: reset every component as well
            this.Clear();
            this.random = new System.Random(this.randomSeed);
            this.Tick = 0;
            this.readyForTicks = false;
        }

        protected void Clear()
        {
            // TODO: reset UI state as well, which probably require the UI all be held in a single state object
            foreach (
                Dictionary<string, WorldObjectCore> worldObjects in this.core.worldObjects.Values
            )
            {
                foreach (WorldObjectCore worldObject in worldObjects.Values)
                {
                    Destroy((worldObject.backref as WorldObject).gameObject);
                }
            }
            this.core.worldObjects.Clear(); //TODO: do we need to set the worldObjects to null?
        }

        protected void Move(GameControllerCore.MovementQueueItem movementQueueItem)
        {
            // Don't try to move an object that doesn't exist or has been deleted
            if (movementQueueItem.worldObject == null)
            {
                return;
            }

            // Remove the object from the old position, if it exists there
            if (this.core.worldObjects.GetValueOrDefault(movementQueueItem.oldPosition) != null)
            {
                this.core.worldObjects[movementQueueItem.oldPosition]
                    .Remove(movementQueueItem.worldObject.guid);
            }
            // Initialize the new position if it doesn't exist, this happens frequently
            if (this.core.worldObjects.GetValueOrDefault(movementQueueItem.newPosition) == null)
            {
                this.core.worldObjects[movementQueueItem.newPosition] =
                    new Dictionary<string, WorldObjectCore>();
            }

            // Add the object to the new position ---

            // --- Movement is a special case in that is needs to act on both the
            // --- WorldObjectCore and the WorldObject. That is, `core` and `backref`.
            WorldObjectCore worldObjectCore = movementQueueItem.worldObject;
            WorldObject worldObject = movementQueueItem.worldObject.backref as WorldObject;

            // --- Put the object in the new position, position indexes on `core`.
            this.core.worldObjects[movementQueueItem.newPosition][worldObject.core.guid] =
                worldObjectCore;

            // --- Tell the object about its new position, this needs to be set on `backref`.
            worldObject.GridPosition = movementQueueItem.newPosition;
            worldObject.SetName();
        }

        protected void Delete(GameControllerCore.DeletionQueueItem deletionQueueItem)
        {
            // Get this position
            Dictionary<string, WorldObjectCore> worldObjects =
                this.core.worldObjects.GetValueOrDefault(deletionQueueItem.position, null);

            // Nothing is here
            if (worldObjects == null)
            {
                return;
            }

            // Find the thing
            WorldObjectCore worldObject = worldObjects.GetValueOrDefault(
                deletionQueueItem.worldObject.guid,
                null
            );

            // Something is here, but not the thing we want
            if (worldObject == null)
            {
                return;
            }

            // Delete the thing
            // Destroy(worldObject.gameObject);
            this.core.worldObjects[deletionQueueItem.position]
                .Remove(deletionQueueItem.worldObject.guid);
            deletionQueueItem.worldObject = null;
        }

        protected virtual void Spawn(SpawnQueueItem spawnQueueItem)
        {
            GameObject thisGameObject = null;
            try
            {
                spawnQueueItem.gridPosition = spawnQueueItem.xyCentered
                    ? new System.Numerics.Vector2(
                        (this.Map.mapSize.x / 2) + spawnQueueItem.x,
                        (this.Map.mapSize.y / 2) + spawnQueueItem.y
                    )
                    : new System.Numerics.Vector2(spawnQueueItem.x, spawnQueueItem.y);

                // TODO: don't allow spawning 2 buildings on the same tile

                // TODO: find spawnables once, then cache
                Transform spawnablesTransform = this.spawnables.transform.Find(spawnQueueItem.type);
                GameObject gameObject = spawnablesTransform.gameObject;
                thisGameObject = Instantiate(gameObject, new Vector2(), Quaternion.identity);

                // TODO: spawn a game object to hold each world object type
                // TODO: assign the thisGameObject to be the child of that world object type
                WorldObject worldObject = thisGameObject.GetComponent<WorldObject>();
                thisGameObject.transform.SetParent(this.Map.WorldGameObject.transform);

                // Instantiate is a custom function on each world object, it only conceptually relates to Unity's Instantiate
                // This base Instantiate function (eg. not PostInstantiate) is responsible for setting simple values like
                // grid position, and initializing "simple" components like the resource component.
                worldObject.Instantiate(spawnQueueItem);

                // PostInstantiate is a custom function on each world object, similar to Instantiate above. It is responsible
                // for setting up more complex components like the status component, and calling the callback function.
                // The exists because the children of WorldObject have their own custom components that need to be initialized,
                // in a particular order. For example the ResourcesComponent needs to be initialized before the StatusComponent.
                worldObject.PostInstantiate(spawnQueueItem);

                // Initialize the dictionary if it doesn't exist, this will only happen once
                this.core.worldObjects ??=
                    new Dictionary<System.Numerics.Vector2, Dictionary<string, WorldObjectCore>>();

                // Initialize the current position if it doesn't exist, this happens frequently
                // Null coallesce doesn't work here, not totally sure why
                if (this.core.worldObjects.GetValueOrDefault(spawnQueueItem.gridPosition) == null)
                {
                    this.core.worldObjects[spawnQueueItem.gridPosition] =
                        new Dictionary<string, WorldObjectCore>();
                }

                // We assume that the GetGuid() is unique enough to not cause a collision here
                this.core.worldObjects[spawnQueueItem.gridPosition][worldObject.Guid] =
                    worldObject.core;
            }
            catch (GameControllerCore.SpawnException ex)
            {
                if (thisGameObject != null)
                {
                    Debug.Log(ex.Message);
                    Destroy(thisGameObject);
                }
                // TODO: write exception message to parent's status
                // TODO: delete the object if you've already spawned it... at any single line in the above code
            }
        }
    }
}
#endif
