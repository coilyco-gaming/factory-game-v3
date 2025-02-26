#if UNITY_6000
namespace Assets.Scripts.Components.Unity
{
    using System;
    using System.Collections.Generic;
    using UnityEngine;

    public class StatusDataComponent : MonoBehaviour
    {
        // PROPERTIES //

        public Func<StatusData> Data { get; set; } = null;

        // CLASSES //

        public class StatusData
        {
            public string Name { get; set; } = null;
            public Dictionary<string, string> Info { get; set; } = null;
            public List<Dictionary<int, string>> Alerts { get; set; } = null;
        }

        public void Instantiate() { }
    }
}
#endif
