using System.Collections.Generic;

namespace Assets.Scripts.ScriptableObject
{
    public class SpawnQueueItem
    {
        public string type;
        public bool xyCentered;
        public int x;
        public int y;
        public System.Numerics.Vector2 gridPosition;
        public string targetType;
        public string targetSubType;
        public Dictionary<string, uint> resources;

        public SpawnQueueItem(
            string type,
            int x,
            int y,
            bool xyCentered = false,
            string targetType = "",
            string targetSubType = "",
            Dictionary<string, uint> resources = null
        )
        {
            this.type = type;
            this.targetType = targetType;
            this.targetSubType = targetSubType;
            this.resources = resources;
            this.xyCentered = xyCentered;
            this.x = x;
            this.y = y;
        }
    }
}
