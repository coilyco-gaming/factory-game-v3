namespace Assets.Scripts.WorldObjects.Core
{
    using System;
    using System.Collections.Generic;
    using Assets.Scripts.Components.Core;
    using Assets.Scripts.Core;

    public class WorldObjectCore
    {
        public object backref;
        private System.Numerics.Vector2 gridPosition;

        // PROPERTIES //

        public float ZIndex => 1;

        public ResourcesComponentCore Resources { get; set; }
        public BatteryComponentCore Battery { get; set; }
        public List<InserterComponentCore> Inserters { get; set; }
        public ProductionComponentCore Production { get; set; }
        public PowerComponentCore Power { get; set; }
        public StatusDataComponentCore Status { get; set; }

        public string Guid { get; set; }

        public string WorldObjectType { get; set; }

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

        public void Instantiate(
            GameControllerCore gameController,
            GameControllerCore.SpawnQueueItem spawnQueueItem
        )
        {
            spawnQueueItem.instantiateCallback?.Invoke(gameController, this);
            this.GridPosition = spawnQueueItem.gridPosition;
            this.Guid = this.CreateGuid();
        }

        public void PostInstantiate(
            GameControllerCore gameController,
            GameControllerCore.SpawnQueueItem spawnQueueItem
        )
        {
            spawnQueueItem.postInstantiateCallback?.Invoke(gameController, this);
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
    using Assets.Scripts.Components.Unity;
    using Assets.Scripts.Core;
    using Assets.Scripts.Unity;
    using Assets.Scripts.WorldObjects.Core;
    using global::Unity.VisualScripting;
    using UnityEngine;

    public class WorldObject : MonoBehaviour
    {
        public WorldObjectCore core;

        // PROPERTIES //

        public virtual float ZIndex => 1;

        public string Guid
        {
            get => this.core.Guid;
            set => this.core.Guid = value;
        }

        public string WorldObjectType
        {
            get => this.core.WorldObjectType;
            set => this.core.WorldObjectType = value;
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

        public virtual void Instantiate(
            GameController gameController,
            GameControllerCore.SpawnQueueItem spawnQueueItem
        )
        {
            this.core = new WorldObjectCore(this);
            this.core.Instantiate(gameController.core, spawnQueueItem);
            this.GridPosition = spawnQueueItem.gridPosition; // This is a special case because it sets the transform position
            this.WorldObjectType = this.transform.name.Replace("(Clone)", "");
            this.SetName();
        }

        public virtual void PostInstantiate(
            GameController gameController,
            GameControllerCore.SpawnQueueItem spawnQueueItem
        )
        {
            this.core.PostInstantiate(gameController.core, spawnQueueItem);
            this.core.Status = new() { Data = this.GetStatusData() };
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
