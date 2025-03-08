namespace Assets.Scripts.WorldObjects.Core
{
    using System;
    using System.Collections.Generic;
    using Assets.Scripts.Components.Core;
    using Assets.Scripts.Core;

    [Serializable]
    public class WorldObjectCore
    {
        // PROPERTIES //

        public float ZIndex => 1;

        // TODO: turn all of these into fields

        // TODO: make each component manage its state via a "data" field on the world object

        // TODO: add odin inspector to all of the serializable classes
        // https://odininspector.com/tutorials

        public MovementComponentCore movement;
        public DispatchComponentCore dispatch;
        public DispatchReceiverComponentCore dispatchReceiver;
        public BatteryComponentCore battery;
        public ResourcesComponentCore resources;
        public List<ResourceInserterComponentCore> resourceInserters;
        public ResourceInserterComponentCore resourceReceiver;
        public ProductionComponentCore production;
        public PowerComponentCore power;
        public string guid;
        public string worldObjectType;
        public string targetType;
        public string targetSubType;
        public object backref;
        public bool mobile = false;
        public bool passThrough = false;
        public System.Numerics.Vector2 gridPosition;

        public System.Numerics.Vector2 GridPosition
        {
            get => this.gridPosition;
            set => this.gridPosition = value;
        }

        // FUNCTIONS //

        public WorldObjectCore(object backref)
        {
            this.backref = backref;
        }

        public void Instantiate(SpawnQueueItem spawnQueueItem, GameContent gameContent)
        {
            this.GridPosition = spawnQueueItem.gridPosition;
            this.targetType = spawnQueueItem.targetType;
            this.targetSubType = spawnQueueItem.targetSubType;
            this.guid = this.CreateGuid();
        }

        public void PostInstantiate(SpawnQueueItem spawnQueueItem, GameContent gameContent)
        {
            if (this.resources == null && spawnQueueItem.resources != null)
            {
                throw new GameControllerCore.MisconfigurationException(
                    $"WorldObject {this.worldObjectType} has no resources component but has resources."
                );
            }
            if (spawnQueueItem.resources != null)
            {
                this.resources.resources = spawnQueueItem.resources;
            }
        }

        public string CreateGuid()
        {
            long time = DateTime.UtcNow.Ticks;
            byte[] guidBytes = System.Guid.NewGuid().ToByteArray();
            byte[] counterBytes = BitConverter.GetBytes(time);
            Array.Copy(counterBytes, 0, guidBytes, guidBytes.Length - 8, 8);
            Guid timeOffsetGuid = new(guidBytes);
            string guidString = timeOffsetGuid.ToString();
            return guidString;
        }
    }
}

namespace Assets.Scripts.WorldObjects.Unity
{
    using System;
    using Assets.Scripts.Components.Core;
    using Assets.Scripts.Core;
    using Assets.Scripts.Unity;
    using Assets.Scripts.WorldObjects.Core;
    using UnityEngine;

    [Serializable]
    public class WorldObject : MonoBehaviour
    {
        public WorldObjectCore core;

        // PROPERTIES //

        public virtual float ZIndex => 1;

        public string Guid
        {
            get => this.core.guid;
            set => this.core.guid = value;
        }

        public string WorldObjectType
        {
            get => this.core.worldObjectType;
            set => this.core.worldObjectType = value;
        }

        public System.Numerics.Vector2 GridPosition
        {
            get => this.core.GridPosition;
            set
            {
                this.core.GridPosition = value;
                this.transform.localPosition = new Vector3(value.X, value.Y, -this.ZIndex);
            }
        }

        public virtual StatusDataComponentCore StatusData =>
            new()
            { //
                Name = Util.HumanizedString(this.WorldObjectType),
            };

        // FUNCTIONS //

        public virtual void Tick(GameController gameController) { }

        public virtual void Instantiate(SpawnQueueItem spawnQueueItem, GameContent gameContent)
        {
            this.core = new WorldObjectCore(this);
            this.core.Instantiate(spawnQueueItem, gameContent);
            this.GridPosition = spawnQueueItem.gridPosition; // This is a special case because it sets the transform position
            this.WorldObjectType = this.transform.name.Replace("(Clone)", "");
            this.SetName();
        }

        public virtual void PostInstantiate(SpawnQueueItem spawnQueueItem, GameContent gameContent)
        {
            this.core.PostInstantiate(spawnQueueItem, gameContent);
        }

        public void SetName()
        {
            this.transform.name =
                $"{this.WorldObjectType} ({this.GridPosition.X}, {this.GridPosition.Y})";
        }
    }
}
