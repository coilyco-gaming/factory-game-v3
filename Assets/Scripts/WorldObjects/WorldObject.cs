using System;
using Assets.Scripts.Components;
using Unity.VisualScripting;
using UnityEngine;

namespace Assets.Scripts.WorldObjects
{
    public class WorldObject : MonoBehaviour
    {
        // FIELDS //

        private System.Numerics.Vector2 gridPosition;

        // PROPERTIES //

        public virtual float ZIndex => 1;

        public virtual float Size => 1;

        public virtual StatusDataComponent Status { get; set; }

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
            this.WorldObjectType = this.transform.name.Replace("(Clone)", "");
            this.GridPosition = spawnQueueItem.gridPosition;
            this.Guid = this.CreateGuid();
            this.SetName();
        }

        public virtual void PostInstantiate(
            GameController gameController,
            GameController.SpawnQueueItem spawnQueueItem
        )
        {
            spawnQueueItem.callback?.Invoke(gameController, this);
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

        protected void FaceMovement(System.Numerics.Vector2 movement)
        {
            float angle = Mathf.Atan2(-movement.X, movement.Y) * Mathf.Rad2Deg;
            Quaternion rotation = Quaternion.Euler(0, 0, angle);
            this.transform.rotation = rotation;
        }

        protected void FaceLocation(System.Numerics.Vector2 target)
        {
            float yOffset = target.Y - this.GridPosition.Y;
            float xOffset = target.X - this.GridPosition.X;
            float angle = Mathf.Atan2(-xOffset, yOffset) * Mathf.Rad2Deg;
            Quaternion rotation = Quaternion.Euler(0, 0, angle);
            this.transform.rotation = rotation;
        }

        protected System.Numerics.Vector2? PathFind(
            GameController gameController,
            System.Numerics.Vector2 target
        )
        {
            return gameController.Map.GetMovement(gameController, this.GridPosition, target);
        }

        private string CreateGuid()
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
