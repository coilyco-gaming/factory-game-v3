using System;
using System.Collections.Generic;
using System.Linq;
using Assets.Scripts.Components.Unity;
using Assets.Scripts.WorldObjects;
using UnityEngine;

namespace Assets.Scripts
{
    public class GameController : MonoBehaviour
    {
        public GameObject spawnables;
        public GameObject mapTiles;
        public GameObject userInterface;
        public float tickFrequency = 0.1f;
        protected int randomSeed = 0;
        protected System.Random random;
        protected bool readyForTicks = false;
        protected int maxTicks = 0;
        private Dictionary<System.Numerics.Vector2, Dictionary<string, WorldObject>> worldObjects =
            new();
        private List<DeletionQueueItem> queuedForDeletion = new();
        private List<SpawnQueueItem> queuedForSpawn = new();
        private List<MovementQueueItem> queuedForMovement = new();
        private float lastTick = 0;

        // PROPERTIES //

        public int Tick { get; protected set; } = 0;
        public SpriteMapComponent Map { get; set; }
        protected PlayerComponent PlayerComponent { get; set; }
        private StatusUIComponent StatusUIComponent { get; set; }

        // CLASSES //

        public class SpawnException : Exception
        {
            public SpawnException(string message)
                : base(message) { }
        }

        public class DeletionQueueItem
        {
            public WorldObject worldObject;
            public System.Numerics.Vector2 position;

            public DeletionQueueItem(WorldObject worldObject, System.Numerics.Vector2 position)
            {
                this.worldObject = worldObject;
                this.position = position;
            }
        }

        public class SpawnQueueItem
        {
            public string name;
            public WorldObject parent;
            public System.Numerics.Vector2 gridPosition;
            public int size;
            public Predicate<GameController> conditions;
            public Action<GameController, WorldObject> callback;

            public SpawnQueueItem(
                string name,
                System.Numerics.Vector2 gridPosition,
                WorldObject parent = null,
                int size = 1,
                Predicate<GameController> conditions = null,
                Action<GameController, WorldObject> callback = null
            )
            {
                this.name = name;
                this.parent = parent;
                this.gridPosition = gridPosition;
                this.size = size;
                this.conditions = conditions;
                this.callback = callback;
            }
        }

        public class MovementQueueItem
        {
            public System.Numerics.Vector2 oldPosition;
            public System.Numerics.Vector2 newPosition;
            public WorldObject worldObject;

            public MovementQueueItem(
                System.Numerics.Vector2 oldPosition,
                System.Numerics.Vector2 newPosition,
                WorldObject worldObject
            )
            {
                this.oldPosition = oldPosition;
                this.newPosition = newPosition;
                this.worldObject = worldObject;
            }
        }

        // FUNCTIONS //

        public virtual void Start()
        {
            this.Map = this.GetComponent<SpriteMapComponent>();
            this.Map.Instantiate(this.GetComponent<Canvas>());

            this.PlayerComponent = this.GetComponent<PlayerComponent>();
            this.PlayerComponent.Instantiate(this.Map.mapSize.x, this.Map.mapSize.y);

            this.StatusUIComponent = this.GetComponent<StatusUIComponent>();
            this.StatusUIComponent.Instantiate(this.userInterface);

            this.Reset();
        }

        public void Update()
        {
            // If we aren't ready for ticks, the main game loop won't run
            if (!this.readyForTicks)
            {
                return;
            }

            // If the max ticks is set and we've reached it, stop the game loop
            if (this.maxTicks != 0 && this.Tick >= this.maxTicks)
            {
                return;
            }

            // This is the main game loop
            if (Time.time > this.lastTick + this.tickFrequency)
            {
                // Update the UI state with whatever the player is looking at
                this.WriteStatusUI();

                // Tick all objects
                foreach (Dictionary<string, WorldObject> worldObjects in this.worldObjects.Values)
                {
                    foreach (WorldObject worldObject in worldObjects.Values)
                    {
                        worldObject.Tick(this);
                    }
                }

                // Delete queued objects
                if (this.queuedForDeletion != null)
                {
                    foreach (DeletionQueueItem deletionQueueItem in this.queuedForDeletion)
                    {
                        this.Delete(deletionQueueItem);
                    }
                    this.queuedForDeletion.Clear();
                }

                // TODO: make sure an object isnt being moved while being deleted.
                // TODO: LERP the movement of objects
                // Move queued objects
                if (this.queuedForMovement != null)
                {
                    foreach (MovementQueueItem movementQueueItem in this.queuedForMovement)
                    {
                        this.Move(movementQueueItem);
                    }
                    this.queuedForMovement.Clear();
                }

                // Spawn queued objects
                if (this.queuedForSpawn != null)
                {
                    foreach (SpawnQueueItem spawnQueueItem in this.queuedForSpawn)
                    {
                        this.Spawn(spawnQueueItem);
                    }
                    this.queuedForSpawn.Clear();
                }

                // Handle ticks
                this.lastTick = Time.time;
                this.Tick++;
            }
        }

        public Dictionary<
            System.Numerics.Vector2,
            Dictionary<string, WorldObject>
        > GetWorldObjects()
        {
            return this.worldObjects;
        }

        public List<WorldObject> GetWorldObjectsByPosition(System.Numerics.Vector2 position)
        {
            Dictionary<string, WorldObject> worldObjects = this.worldObjects.GetValueOrDefault(
                position,
                null
            );

            return worldObjects?.Values.ToList();
        }

        public List<WorldObject> GetWorldObjectsByPositionAndType(
            System.Numerics.Vector2 position,
            List<string> types
        )
        {
            List<WorldObject> worldObjects = this.GetWorldObjectsByPosition(position);

            // Nothing is here
            if (worldObjects == null)
            {
                return null;
            }

            // Find the things that match the type
            List<WorldObject> matchingWorldObjects = new();
            foreach (WorldObject worldObject in worldObjects)
            {
                if (types.Contains(worldObject.WorldObjectType))
                {
                    matchingWorldObjects.Add(worldObject);
                }
            }
            return matchingWorldObjects;
        }

        public void QueueForMovement(MovementQueueItem movementQueueItem)
        {
            this.queuedForMovement ??= new List<MovementQueueItem>();
            this.queuedForMovement.Add(movementQueueItem);
        }

        public void QueueForDeletion(DeletionQueueItem deletionQueueItem)
        {
            this.queuedForDeletion ??= new List<DeletionQueueItem>();
            this.queuedForDeletion.Add(deletionQueueItem);
        }

        public void QueueForSpawn(SpawnQueueItem spawnQueueItem)
        {
            this.queuedForSpawn ??= new List<SpawnQueueItem>();
            this.queuedForSpawn.Add(spawnQueueItem);
        }

        protected virtual void Reset()
        {
            // TODO: reset every component as well
            this.Clear();
            this.PlayerComponent.Reset();
            this.random = new System.Random(this.randomSeed);
            this.Tick = 0;
            this.readyForTicks = false;
        }

        protected void Clear()
        {
            // TODO: reset UI state as well, which probably require the UI all be held in a single state object
            foreach (Dictionary<string, WorldObject> worldObjects in this.worldObjects.Values)
            {
                foreach (WorldObject worldObject in worldObjects.Values)
                {
                    Destroy(worldObject.gameObject);
                }
            }
            this.worldObjects.Clear();
        }

        protected void Move(MovementQueueItem movementQueueItem)
        {
            // Don't try to move an object that doesn't exist or has been deleted
            if (movementQueueItem.worldObject == null)
            {
                return;
            }
            // Remove the object from the old position, if it exists there
            if (this.worldObjects.GetValueOrDefault(movementQueueItem.oldPosition) != null)
            {
                this.worldObjects[movementQueueItem.oldPosition]
                    .Remove(movementQueueItem.worldObject.Guid);
            }
            // Initialize the new position if it doesn't exist, this happens frequently
            if (this.worldObjects.GetValueOrDefault(movementQueueItem.newPosition) == null)
            {
                this.worldObjects[movementQueueItem.newPosition] =
                    new Dictionary<string, WorldObject>();
            }
            // Add the object to the new position
            this.worldObjects[movementQueueItem.newPosition][movementQueueItem.worldObject.Guid] =
                movementQueueItem.worldObject;
            movementQueueItem.worldObject.GridPosition = movementQueueItem.newPosition;
            movementQueueItem.worldObject.SetName();
        }

        protected void Delete(DeletionQueueItem deletionQueueItem)
        {
            // Get this position
            Dictionary<string, WorldObject> worldObjects = this.worldObjects.GetValueOrDefault(
                deletionQueueItem.position,
                null
            );

            // Nothing is here
            if (worldObjects == null)
            {
                return;
            }

            // Find the thing
            WorldObject worldObject = worldObjects.GetValueOrDefault(
                deletionQueueItem.worldObject.Guid,
                null
            );

            // Something is here, but not the thing we want
            if (worldObject == null)
            {
                return;
            }

            // Delete the thing
            Destroy(worldObject.gameObject);
            this.worldObjects[deletionQueueItem.position]
                .Remove(deletionQueueItem.worldObject.Guid);
            deletionQueueItem.worldObject = null;
        }

        protected void Spawn(SpawnQueueItem spawnQueueItem)
        {
            GameObject thisGameObject = null;
            try
            {
                // If spawn conditions are set and aren't met, don't spawn
                if (spawnQueueItem.conditions != null)
                {
                    if (!spawnQueueItem.conditions.Invoke(this))
                    {
                        // TODO: write to UI state as an error message
                        return;
                    }
                }

                // Spawning the object is a complex multi-step process!
                Transform spawnablesTransform = this.spawnables.transform.Find(spawnQueueItem.name); // TODO: find objects once, then cache
                GameObject gameObject = spawnablesTransform.gameObject;
                thisGameObject = Instantiate(gameObject, new Vector2(), Quaternion.identity);
                thisGameObject.transform.SetParent(this.Map.WorldGameObject.transform);
                WorldObject worldObject = thisGameObject.GetComponent<WorldObject>();

                // Initialize the object with its custom code

                // Instantiate is a custom function on each world object, it only conceptaully relates to Unity's Instantiate
                // This base Instantiate function (eg. not PostInstantiate) is responsible for setting simple values like
                // grid position, and initializing "simple" components like the resource component.
                worldObject.Instantiate(this, spawnQueueItem);

                // PostInstantiate is a custom function on each world object, similar to Instantiate above. It is responsible
                // for setting up more complex components like the status component, and calling the callback function.
                // The exists because the children of WorldObject have their own custom components that need to be initialized,
                // in a particular order. For example the ResourcesComponent needs to be initialized before the StatusComponent.
                worldObject.PostInstantiate(this, spawnQueueItem);

                // Initialize the dictionary if it doesn't exist, this will only happen once
                this.worldObjects ??=
                    new Dictionary<System.Numerics.Vector2, Dictionary<string, WorldObject>>();

                // Initialize the current position if it doesn't exist, this happens frequently
                // Null coallesce doesn't work here, not totally sure why
                if (this.worldObjects.GetValueOrDefault(spawnQueueItem.gridPosition) == null)
                {
                    this.worldObjects[spawnQueueItem.gridPosition] =
                        new Dictionary<string, WorldObject>();
                }

                // Be chaotic and assume that the GetGuid() is unique
                this.worldObjects[spawnQueueItem.gridPosition][worldObject.Guid] = worldObject;
            }
            catch (SpawnException ex)
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

        private void WriteStatusUI()
        {
            // Get the list of objects at the player's position
            System.Numerics.Vector2 position = this.PlayerComponent.GetGridPosition();

            // TODO: grab the nearby objects, not just the ones at the player's position
            List<WorldObject> worldObjects =
                this.worldObjects.GetValueOrDefault(position, null)?.Values.ToList()
                ?? new List<WorldObject>();

            // Display the status data in the UI
            this.StatusUIComponent.Display(worldObjects);
        }
    }
}
