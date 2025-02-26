namespace Assets.Scripts.WorldObjects.Unity
{
    using System;
    using Assets.Scripts.Components.Unity;
    using Assets.Scripts.Unity;
    using global::Unity.VisualScripting;
    using UnityEngine;

    public class WorldObject : MonoBehaviour
    {
        private System.Numerics.Vector2 gridPosition;

        // PROPERTIES //

        public virtual float ZIndex => 1;

        public virtual StatusDataComponent Status { get; set; }
        public ResourcesComponent Resources { get; set; }
        public BatteryComponent Battery { get; set; }

        public string Guid { get; set; }

        public string WorldObjectType { get; set; }

        public System.Numerics.Vector2 GridPosition
        {
            get => this.gridPosition;
            set
            {
                this.gridPosition = value;
                this.transform.localPosition = new Vector3(value.X, value.Y, -this.ZIndex);
            }
        }

        // FUNCTIONS //

        public virtual void Tick(GameController gameController) { }

        public virtual void Instantiate(
            GameController gameController,
            GameController.SpawnQueueItem spawnQueueItem
        )
        {
            spawnQueueItem.instantiateCallback?.Invoke(gameController, this);
            this.GridPosition = spawnQueueItem.gridPosition;
            this.Guid = this.CreateGuid();
            // this.SetName();
            this.Resources = this.AddComponent<ResourcesComponent>();
            this.Battery = this.AddComponent<BatteryComponent>();
            this.WorldObjectType = this.transform.name.Replace("(Clone)", "");
        }

        public virtual void PostInstantiate(
            GameController gameController,
            GameController.SpawnQueueItem spawnQueueItem
        )
        {
            spawnQueueItem.postInstantiateCallback?.Invoke(gameController, this);
            this.Status = this.AddComponent<StatusDataComponent>();
            this.Status.Instantiate();
            this.Status.Data = this.GetStatusData();
        }

        public void MoveTo(GameController gameController, System.Numerics.Vector2 movement)
        {
            System.Numerics.Vector2 newPosition = new(
                this.GridPosition.X + movement.X,
                this.GridPosition.Y + movement.Y
            );
            gameController.QueueForMovement(
                new GameController.MovementQueueItem(this.GridPosition, newPosition, this)
            );
        }

        public void SetName()
        {
            this.transform.name =
                $"{this.WorldObjectType} ({this.GridPosition.X}, {this.GridPosition.Y})";
        }

        protected virtual Func<StatusDataComponent.StatusData> GetStatusData()
        {
            return () => new StatusDataComponent.StatusData() { Name = this.WorldObjectType };
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
