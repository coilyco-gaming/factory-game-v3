namespace Assets.Scripts.WorldObjects.Core
{
    using System;
    using System.Collections.Generic;
    using Assets.Scripts.Components.Core;
    using Assets.Scripts.ScriptableObject;

    [Serializable]
    public class WorldObjectCore
    {
        // PROPERTIES //

        public float ZIndex => 1;

        // TODO: turn all of these into fields

        // TODO: make each component manage its state via a "data" field on the world object

        // TODO: add odin inspector to all of the serializable classes
        // https://odininspector.com/tutorials

        public DispatchComponentCore dispatch;
        public DispatchReceiverComponentCore receiver;
        public TransferHubComponent transferHub;
        public ResourcesComponentCore resources;
        public BatteryComponentCore battery;
        public List<InserterComponentCore> inserters;
        public ProductionComponentCore production;
        public PowerComponentCore power;
        public StatusDataComponentCore status;
        public string guid;
        public string worldObjectType;
        public string targetType;
        public string targetSubType;
        public object backref;
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

        public void Instantiate(SpawnQueueItem spawnQueueItem)
        {
            this.GridPosition = spawnQueueItem.gridPosition;
            this.targetType = spawnQueueItem.targetType;
            this.guid = this.CreateGuid();
        }

        public void PostInstantiate(SpawnQueueItem spawnQueueItem)
        {
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

#if UNITY_6000
namespace Assets.Scripts.WorldObjects.Unity
{
    using System;
    using Assets.Scripts.Components.Core;
    using Assets.Scripts.Core;
    using Assets.Scripts.ScriptableObject;
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

        // FUNCTIONS //

        public virtual void Tick(GameController gameController) { }

        public virtual void Instantiate(SpawnQueueItem spawnQueueItem)
        {
            this.core = new WorldObjectCore(this);
            this.core.Instantiate(spawnQueueItem);
            this.GridPosition = spawnQueueItem.gridPosition; // This is a special case because it sets the transform position
            this.WorldObjectType = this.transform.name.Replace("(Clone)", "");
            this.SetName();
        }

        public virtual void PostInstantiate(SpawnQueueItem spawnQueueItem)
        {
            this.core.PostInstantiate(spawnQueueItem);
            this.core.status = new() { Data = this.GetStatusData() };
        }

        public void MoveTo(GameController gameController, System.Numerics.Vector2 movement)
        {
            System.Numerics.Vector2 newPosition = new(
                this.GridPosition.X + movement.X,
                this.GridPosition.Y + movement.Y
            );
            gameController.QueueForMovement(
                new GameControllerCore.MovementQueueItem(this.GridPosition, newPosition, this.core)
            );
        }

        public void SetName()
        {
            this.transform.name =
                $"{this.WorldObjectType} ({this.GridPosition.X}, {this.GridPosition.Y})";
        }

        protected virtual Func<StatusDataComponentCore.StatusData> GetStatusData()
        {
            return () => new StatusDataComponentCore.StatusData() { Name = this.WorldObjectType };
        }
    }
}
#endif
