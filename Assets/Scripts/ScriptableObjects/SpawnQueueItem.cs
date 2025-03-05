using Sirenix.OdinInspector;
using UnityEngine;
using UnityEngine.Rendering;

namespace Assets.Scripts.ScriptableObject
{
    [CreateAssetMenu]
    public class SpawnQueueItem : SerializedScriptableObject
    {
        public string type;
        public bool xyCentered;
        public int x;
        public int y;
        public System.Numerics.Vector2 gridPosition;
        public string targetType;
        public SerializedDictionary<string, uint> resources;

        public SpawnQueueItem(
            string type,
            int x,
            int y,
            bool xyCentered = false,
            string targetType = "",
            SerializedDictionary<string, uint> resources = null
        )
        {
            this.type = type;
            this.targetType = targetType;
            this.resources = resources;
            this.xyCentered = xyCentered;
            this.x = x;
            this.y = y;
        }
    }
}
